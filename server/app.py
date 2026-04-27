"""Serveur web Lelins — interface navigateur pour le générateur d'images.

Lancer avec ``python scripts/serve.py``. L'interface est dispo sur
http://127.0.0.1:8000 (par défaut). Pour y accéder depuis un téléphone ou un
autre appareil, lancer avec ``--host 0.0.0.0`` (LAN) ou via Tailscale.

Le pipeline SDXL est chargé **paresseusement** (à la première requête) pour
que le serveur démarre rapidement.
"""

from __future__ import annotations

import io
import json
import random
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Optional

from fastapi import FastAPI, File, Form, HTTPException, Request, UploadFile
from fastapi.responses import FileResponse, HTMLResponse, JSONResponse
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates


# ---- Chemins ----

ROOT = Path(__file__).resolve().parent.parent
SCRIPTS_DIR = ROOT / "scripts"
CHARACTERS_DIR = ROOT / "characters"
OUTPUTS_DIR = ROOT / "outputs"
SERVER_DIR = Path(__file__).resolve().parent
TEMPLATES_DIR = SERVER_DIR / "templates"
STATIC_DIR = SERVER_DIR / "static"

OUTPUTS_DIR.mkdir(parents=True, exist_ok=True)
(OUTPUTS_DIR / "uploads").mkdir(parents=True, exist_ok=True)

# Permettre l'import des modules `scripts/...`
sys.path.insert(0, str(SCRIPTS_DIR))

from prompts import Character, ProductPrompt, DEFAULT_NEGATIVE  # noqa: E402


# ---- Cache des pipelines (chargement paresseux) ----

_pipe_text2img = None
_pipe_inpaint = None
_face_adapter_loaded = False
_general_adapter_loaded_on_inpaint = False


def get_text2img_pipe():
    """Charge SDXL text-to-image au premier usage, puis le réutilise."""
    global _pipe_text2img
    if _pipe_text2img is None:
        from generate import DEFAULT_MODEL_ID, build_pipeline, detect_device
        device, dtype = detect_device()
        print(f"[serveur] Chargement SDXL base sur {device}…", flush=True)
        t0 = time.time()
        _pipe_text2img = build_pipeline(DEFAULT_MODEL_ID, device, dtype)
        print(f"[serveur] SDXL base prêt en {time.time() - t0:.1f}s.", flush=True)
    return _pipe_text2img


def ensure_face_adapter(scale: float = 0.7):
    """Charge l'IP-Adapter face sur le pipeline text2img si pas déjà fait."""
    global _face_adapter_loaded
    pipe = get_text2img_pipe()
    if not _face_adapter_loaded:
        from ip_adapter_helpers import load_face_adapter
        print("[serveur] Chargement IP-Adapter face…", flush=True)
        load_face_adapter(pipe, scale=scale)
        _face_adapter_loaded = True
    else:
        pipe.set_ip_adapter_scale(scale)
    return pipe


def get_inpaint_pipe_with_general_adapter(scale: float = 0.85):
    """Charge SDXL inpainting + IP-Adapter général au premier usage."""
    global _pipe_inpaint, _general_adapter_loaded_on_inpaint
    if _pipe_inpaint is None:
        from tryon import DEFAULT_INPAINT_MODEL_ID, build_inpaint_pipeline, detect_device
        device, dtype = detect_device()
        print(f"[serveur] Chargement SDXL Inpainting sur {device}…", flush=True)
        t0 = time.time()
        _pipe_inpaint = build_inpaint_pipeline(DEFAULT_INPAINT_MODEL_ID, device, dtype)
        print(f"[serveur] SDXL Inpainting prêt en {time.time() - t0:.1f}s.", flush=True)
    if not _general_adapter_loaded_on_inpaint:
        from ip_adapter_helpers import load_general_adapter
        print("[serveur] Chargement IP-Adapter général sur inpaint…", flush=True)
        load_general_adapter(_pipe_inpaint, scale=scale)
        _general_adapter_loaded_on_inpaint = True
    else:
        _pipe_inpaint.set_ip_adapter_scale(scale)
    return _pipe_inpaint


def detect_device_safe():
    """Détecte le matériel sans charger de modèle."""
    try:
        from generate import detect_device
        return detect_device()
    except Exception as e:
        return ("?", f"err:{e}")


# ---- Helpers ----

def _make_torch_generator(seed: int):
    import torch
    device, _ = detect_device_safe()
    g = torch.Generator(device="cpu" if device in ("mps", "?") else device)
    return g.manual_seed(seed)


def _timestamp_name(prefix: str, ext: str = "png") -> str:
    return f"{prefix}-{datetime.now().strftime('%Y%m%d-%H%M%S')}-{random.randint(1000, 9999)}.{ext}"


def _list_characters() -> list[dict]:
    """Liste les profils JSON dans characters/ (en excluant les *.example.json)."""
    if not CHARACTERS_DIR.exists():
        return []
    out = []
    for f in sorted(CHARACTERS_DIR.glob("*.json")):
        if f.name.endswith(".example.json"):
            continue
        try:
            data = json.loads(f.read_text(encoding="utf-8"))
            out.append({
                "path": str(f.relative_to(ROOT)),
                "name": data.get("name", f.stem),
                "has_portrait": bool(data.get("portrait_path")),
                "seed": data.get("seed"),
            })
        except Exception as e:
            out.append({"path": str(f.relative_to(ROOT)), "name": f.stem, "error": str(e)})
    return out


# ---- App FastAPI ----

app = FastAPI(title="Lelins Image Generator")
templates = Jinja2Templates(directory=str(TEMPLATES_DIR))

if STATIC_DIR.exists():
    app.mount("/static", StaticFiles(directory=str(STATIC_DIR)), name="static")
app.mount("/outputs", StaticFiles(directory=str(OUTPUTS_DIR)), name="outputs")


@app.get("/", response_class=HTMLResponse)
def index(request: Request):
    return templates.TemplateResponse(
        "index.html",
        {
            "request": request,
            "characters": _list_characters(),
            "device": detect_device_safe()[0],
        },
    )


@app.get("/api/status")
def api_status():
    return {
        "device": detect_device_safe()[0],
        "characters": _list_characters(),
        "text2img_loaded": _pipe_text2img is not None,
        "inpaint_loaded": _pipe_inpaint is not None,
        "face_adapter_loaded": _face_adapter_loaded,
    }


@app.post("/api/generate")
def api_generate(
    garment: str = Form(...),
    character_path: Optional[str] = Form(None),
    model_style: str = Form("athletic"),
    background: str = Form("studio_white"),
    pose: str = Form("standing front view, arms relaxed at sides"),
    width: int = Form(1024),
    height: int = Form(1024),
    steps: int = Form(30),
    cfg: float = Form(7.0),
    seed: Optional[int] = Form(None),
    face_scale: float = Form(0.7),
    no_face_lock: bool = Form(False),
):
    """Génère un visuel produit. Charge SDXL au premier appel."""
    character = None
    face_ref_image = None

    if character_path:
        path = ROOT / character_path
        if not path.exists():
            raise HTTPException(404, f"Character introuvable : {character_path}")
        character = Character.from_json_file(path)

    positive, negative = ProductPrompt(
        garment=garment,
        character=character,
        model_style=model_style,
        background=background,
        pose=pose,
    ).build()

    if seed is None and character is not None and character.seed is not None:
        seed = character.seed
    if seed is None:
        seed = random.randint(0, 2**31 - 1)

    use_face_lock = (not no_face_lock) and character is not None and character.portrait_path
    if use_face_lock:
        portrait = ROOT / character.portrait_path
        if not portrait.is_absolute() and not portrait.exists():
            portrait = Path(character.portrait_path)
        if portrait.exists():
            from ip_adapter_helpers import open_reference_image
            pipe = ensure_face_adapter(scale=face_scale)
            face_ref_image = open_reference_image(portrait)
        else:
            pipe = get_text2img_pipe()
    else:
        pipe = get_text2img_pipe()

    generator = _make_torch_generator(seed)

    print(f"[serveur] Génération seed={seed} face_lock={face_ref_image is not None}…", flush=True)
    t0 = time.time()
    pipe_kwargs = dict(
        prompt=positive,
        negative_prompt=negative,
        width=width,
        height=height,
        num_inference_steps=steps,
        guidance_scale=cfg,
        generator=generator,
    )
    if face_ref_image is not None:
        pipe_kwargs["ip_adapter_image"] = face_ref_image
    image = pipe(**pipe_kwargs).images[0]
    elapsed = time.time() - t0

    name = _timestamp_name("gen")
    out_path = OUTPUTS_DIR / name
    image.save(out_path)
    print(f"[serveur] Image générée en {elapsed:.1f}s → {out_path}", flush=True)

    return {
        "image_url": f"/outputs/{name}",
        "seed": seed,
        "elapsed_seconds": round(elapsed, 1),
        "prompt": positive,
        "face_lock": face_ref_image is not None,
    }


@app.post("/api/generate-character")
def api_generate_character(
    profile_path: str = Form(...),
    reroll: bool = Form(False),
    seed: Optional[int] = Form(None),
    width: int = Form(896),
    height: int = Form(1152),
    steps: int = Form(35),
    cfg: float = Form(7.0),
):
    """Génère le portrait d'un Character. Met à jour seed + portrait_path."""
    path = ROOT / profile_path
    if not path.exists():
        raise HTTPException(404, f"Profil introuvable : {profile_path}")

    character = Character.from_json_file(path)

    if seed is not None:
        used_seed = seed
    elif reroll or character.seed is None:
        used_seed = random.randint(0, 2**31 - 1)
    else:
        used_seed = character.seed

    pipe = get_text2img_pipe()
    generator = _make_torch_generator(used_seed)

    print(f"[serveur] Portrait {character.name} seed={used_seed}…", flush=True)
    t0 = time.time()
    image = pipe(
        prompt=character.build_portrait_prompt(),
        negative_prompt=DEFAULT_NEGATIVE,
        width=width,
        height=height,
        num_inference_steps=steps,
        guidance_scale=cfg,
        generator=generator,
    ).images[0]
    elapsed = time.time() - t0

    out_name = f"{path.stem}.png"
    out_path = CHARACTERS_DIR / out_name
    image.save(out_path)
    print(f"[serveur] Portrait sauvé en {elapsed:.1f}s → {out_path}", flush=True)

    if character.seed != used_seed or character.portrait_path != str(out_path):
        character.seed = used_seed
        character.portrait_path = str(out_path)
        character.to_json(path)

    return {
        "image_url": f"/characters/{out_name}",
        "seed": used_seed,
        "elapsed_seconds": round(elapsed, 1),
        "portrait_path": str(out_path.relative_to(ROOT)),
    }


@app.post("/api/tryon")
async def api_tryon(
    person: UploadFile = File(...),
    garment: UploadFile = File(...),
    mask: Optional[UploadFile] = File(None),
    auto_mask: str = Form("none"),  # "none" | "hip" | "torso"
    prompt: str = Form(...),
    garment_scale: float = Form(0.85),
    strength: float = Form(0.95),
    steps: int = Form(35),
    cfg: float = Form(7.5),
    seed: Optional[int] = Form(None),
):
    """Essayage virtuel : applique l'image vêtement sur la person dans la zone masquée."""
    from PIL import Image, ImageDraw, ImageFilter
    from ip_adapter_helpers import open_reference_image

    upload_dir = OUTPUTS_DIR / "uploads"
    upload_dir.mkdir(parents=True, exist_ok=True)

    person_path = upload_dir / _timestamp_name("person")
    garment_path = upload_dir / _timestamp_name("garment", "jpg")

    person_path.write_bytes(await person.read())
    garment_path.write_bytes(await garment.read())

    person_image = Image.open(person_path).convert("RGB")
    w, h = person_image.size
    garment_image = open_reference_image(garment_path)

    # Masque : fourni, ou auto-généré.
    if mask is not None:
        mask_path = upload_dir / _timestamp_name("mask")
        mask_path.write_bytes(await mask.read())
        mask_image = Image.open(mask_path).convert("L")
        if mask_image.size != (w, h):
            mask_image = mask_image.resize((w, h), Image.NEAREST)
    elif auto_mask in ("hip", "torso"):
        mask_image = Image.new("L", (w, h), 0)
        draw = ImageDraw.Draw(mask_image)
        if auto_mask == "hip":
            x1, y1, x2, y2 = int(0.20 * w), int(0.50 * h), int(0.80 * w), int(0.78 * h)
        else:  # torso
            x1, y1, x2, y2 = int(0.18 * w), int(0.18 * h), int(0.82 * w), int(0.55 * h)
        draw.rectangle([x1, y1, x2, y2], fill=255)
        mask_image = mask_image.filter(ImageFilter.GaussianBlur(radius=10))
    else:
        raise HTTPException(400, "Fournir un masque OU auto_mask=hip|torso")

    pipe = get_inpaint_pipe_with_general_adapter(scale=garment_scale)

    if seed is None:
        seed = random.randint(0, 2**31 - 1)
    generator = _make_torch_generator(seed)

    full_prompt = (
        f"{prompt}, high detail fabric texture, realistic clothing, "
        "natural skin tone, seamless integration, photorealistic, 8k, sharp focus"
    )

    print(f"[serveur] Try-on seed={seed}…", flush=True)
    t0 = time.time()
    image = pipe(
        prompt=full_prompt,
        negative_prompt=DEFAULT_NEGATIVE,
        image=person_image,
        mask_image=mask_image,
        ip_adapter_image=garment_image,
        width=w,
        height=h,
        num_inference_steps=steps,
        guidance_scale=cfg,
        strength=strength,
        generator=generator,
    ).images[0]
    elapsed = time.time() - t0

    name = _timestamp_name("tryon")
    out_path = OUTPUTS_DIR / name
    image.save(out_path)
    print(f"[serveur] Try-on terminé en {elapsed:.1f}s → {out_path}", flush=True)

    return {
        "image_url": f"/outputs/{name}",
        "seed": seed,
        "elapsed_seconds": round(elapsed, 1),
    }


@app.get("/characters/{filename}")
def get_character_file(filename: str):
    """Sert les portraits de mannequin enregistrés dans characters/."""
    p = CHARACTERS_DIR / filename
    if not p.exists():
        raise HTTPException(404)
    return FileResponse(p)
