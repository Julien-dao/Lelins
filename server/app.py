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
import traceback
from datetime import datetime
from pathlib import Path
from queue import Empty, Queue
from threading import Thread
from typing import Optional

from fastapi import FastAPI, File, Form, HTTPException, Request, UploadFile
from fastapi.responses import FileResponse, HTMLResponse, JSONResponse, StreamingResponse
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
_pipe_text2img_mode = None  # "standard" (SDXL base) | "fast" (SDXL Turbo)
_pipe_inpaint = None
_face_adapter_loaded = False
_general_adapter_loaded_on_inpaint = False


def _free_memory():
    """Libère la mémoire GPU/MPS après un déchargement de pipeline."""
    import gc
    gc.collect()
    try:
        import torch
        if hasattr(torch, "mps") and hasattr(torch.mps, "empty_cache"):
            torch.mps.empty_cache()
        elif torch.cuda.is_available():
            torch.cuda.empty_cache()
    except Exception:
        pass


def get_text2img_pipe(mode: str = "standard"):
    """Charge SDXL text-to-image au premier usage, ou bascule de mode si besoin.

    mode='standard' → SDXL base (qualité standard, 20-30 pas)
    mode='fast'     → SDXL Turbo (4 pas, cfg=0, ~3-5× plus rapide sur Mac MPS)

    Sur 8 Go de RAM, garder les deux modèles en mémoire est exclu : on
    décharge l'ancien avant de charger le nouveau (lent au moment du switch).
    """
    global _pipe_text2img, _pipe_text2img_mode, _face_adapter_loaded

    if _pipe_text2img is not None and _pipe_text2img_mode != mode:
        print(f"[serveur] Bascule mode {_pipe_text2img_mode} → {mode}, "
              f"libération RAM…", flush=True)
        del _pipe_text2img
        _pipe_text2img = None
        _face_adapter_loaded = False  # adapter à recharger avec le nouveau pipe
        _free_memory()

    if _pipe_text2img is None:
        from generate import (DEFAULT_MODEL_ID, SDXL_TURBO_MODEL_ID,
                              build_pipeline, detect_device)
        device, dtype = detect_device()
        model_id = SDXL_TURBO_MODEL_ID if mode == "fast" else DEFAULT_MODEL_ID
        label = "SDXL Turbo (mode rapide)" if mode == "fast" else "SDXL base"
        print(f"[serveur] Chargement {label} sur {device}…", flush=True)
        t0 = time.time()
        _pipe_text2img = build_pipeline(model_id, device, dtype)
        _pipe_text2img_mode = mode
        print(f"[serveur] {label} prêt en {time.time() - t0:.1f}s.", flush=True)
    return _pipe_text2img


def ensure_face_adapter(scale: float = 0.7, mode: str = "standard"):
    """Charge l'IP-Adapter face sur le pipeline text2img si pas déjà fait."""
    global _face_adapter_loaded
    pipe = get_text2img_pipe(mode=mode)
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


# ----------------------------------------------------------------------------
# Endpoints streaming (Server-Sent Events) — feedback temps réel pour l'UI.
#
# Chaque endpoint démarre la génération dans un thread et retourne un flux SSE.
# Évènements émis :
#   - status   : message d'avancement texte (chargement modèle, etc.)
#   - ready    : modèle prêt, génération sur le point de démarrer
#   - step     : un pas de diffusion terminé (step, total, elapsed, eta)
#   - done     : image prête (image_url, seed, elapsed_seconds, ...)
#   - error    : exception levée (message)
# ----------------------------------------------------------------------------

def _is_oom(exc: Exception) -> bool:
    """Détecte une OOM MPS / CUDA."""
    msg = str(exc).lower()
    return ("out of memory" in msg
            or "mps backend out of memory" in msg
            or "cuda out of memory" in msg)


def _format_sse(event_type: str, **data) -> str:
    payload = {"type": event_type, **data}
    return f"data: {json.dumps(payload)}\n\n"


def _drain_queue_to_sse(queue: Queue, timeout: float = 1800.0):
    """Générateur SSE : lit la queue jusqu'à 'done' ou 'error'."""
    while True:
        try:
            event_type, data = queue.get(timeout=timeout)
        except Empty:
            yield _format_sse("error", message="Timeout (>30 min sans nouvelle)")
            return
        yield _format_sse(event_type, **data)
        if event_type in ("done", "error"):
            return


def _generate_with_oom_retry(
    pipe,
    queue: Queue,
    pipe_kwargs: dict,
    t_ref: Optional[dict] = None,
    fallback_size: int = 512,
):
    """Lance pipe(...) en réessayant à ``fallback_size`` si OOM.

    En 8 Go RAM, certaines combinaisons (1024×1024 + IP-Adapter + adapter
    général) franchissent le plafond MPS. Plutôt que de planter, on libère
    et on retente à 512×512 en informant l'utilisateur.
    """
    try:
        return pipe(**pipe_kwargs).images[0]
    except RuntimeError as e:
        if not _is_oom(e):
            raise
        original = (pipe_kwargs.get("width"), pipe_kwargs.get("height"))
        new_w = min(pipe_kwargs.get("width", fallback_size), fallback_size)
        new_h = min(pipe_kwargs.get("height", fallback_size), fallback_size)
        if (new_w, new_h) == original:
            # Déjà à la résolution mini, rien à essayer.
            raise
        queue.put(("status", {
            "message": f"Mémoire insuffisante à {original[0]}×{original[1]}, "
                       f"on réessaie à {new_w}×{new_h}…"
        }))
        _free_memory()
        pipe_kwargs["width"] = new_w
        pipe_kwargs["height"] = new_h
        if t_ref is not None:
            t_ref["t0"] = time.time()  # reset ETA pour cette nouvelle tentative
        return pipe(**pipe_kwargs).images[0]


def _make_step_callback(queue: Queue, total_steps: int, t_start_ref: dict):
    """Crée un callback diffusers compatible callback_on_step_end."""
    def callback(pipe, step_idx: int, timestep, callback_kwargs):
        step = step_idx + 1
        elapsed = time.time() - t_start_ref["t0"]
        rate = elapsed / step if step > 0 else 0
        eta = rate * (total_steps - step)
        queue.put(("step", {
            "step": step,
            "total": total_steps,
            "elapsed": round(elapsed, 1),
            "eta": round(eta, 1),
            "percent": round(100 * step / total_steps, 1),
        }))
        return callback_kwargs
    return callback


@app.post("/api/generate-stream")
def api_generate_stream(
    garment: str = Form(...),
    character_path: Optional[str] = Form(None),
    model_style: str = Form("athletic"),
    background: str = Form("studio_white"),
    pose: str = Form("standing front view, arms relaxed at sides"),
    width: int = Form(1024),
    height: int = Form(1024),
    steps: int = Form(25),
    cfg: float = Form(7.0),
    seed: Optional[int] = Form(None),
    face_scale: float = Form(0.7),
    no_face_lock: bool = Form(False),
    mode: str = Form("standard"),  # "standard" | "fast"
):
    queue: Queue = Queue()

    # En mode rapide (SDXL Turbo), on impose les paramètres recommandés.
    # SDXL Turbo est entraîné en 512×512 ; au-delà la qualité chute.
    if mode == "fast":
        steps = 4
        cfg = 0.0
        width = min(width, 768)
        height = min(height, 768)

    def emit(event_type, **data):
        queue.put((event_type, data))

    def run():
        try:
            character = None
            if character_path:
                path = ROOT / character_path
                if not path.exists():
                    emit("error", message=f"Character introuvable : {character_path}")
                    return
                character = Character.from_json_file(path)

            positive, negative = ProductPrompt(
                garment=garment,
                character=character,
                model_style=model_style,
                background=background,
                pose=pose,
            ).build()

            used_seed = seed
            if used_seed is None and character is not None and character.seed is not None:
                used_seed = character.seed
            if used_seed is None:
                used_seed = random.randint(0, 2**31 - 1)

            need_load = _pipe_text2img is None or _pipe_text2img_mode != mode
            if need_load:
                if mode == "fast":
                    emit("status", message="Chargement de SDXL Turbo (mode rapide)… "
                         "(premier lancement : téléchargement ~7 Go, peut durer 5-15 min)")
                else:
                    emit("status", message="Chargement de SDXL base… (premier lancement : "
                         "téléchargement ~7 Go, peut durer 5-15 min)")
            pipe = get_text2img_pipe(mode=mode)

            face_ref_image = None
            if (not no_face_lock) and character is not None and character.portrait_path:
                portrait = ROOT / character.portrait_path
                if not portrait.is_absolute() and not portrait.exists():
                    portrait = Path(character.portrait_path)
                if portrait.exists():
                    from ip_adapter_helpers import open_reference_image
                    if not _face_adapter_loaded:
                        emit("status", message="Chargement IP-Adapter face… (~2 Go au premier run)")
                    pipe = ensure_face_adapter(scale=face_scale, mode=mode)
                    face_ref_image = open_reference_image(portrait)

            generator = _make_torch_generator(used_seed)

            emit("ready", message="Génération en cours…", total=steps,
                 face_lock=face_ref_image is not None, mode=mode)

            t_ref = {"t0": time.time()}
            cb = _make_step_callback(queue, steps, t_ref)

            pipe_kwargs = dict(
                prompt=positive,
                negative_prompt=negative if mode != "fast" else None,
                width=width,
                height=height,
                num_inference_steps=steps,
                guidance_scale=cfg,
                generator=generator,
                callback_on_step_end=cb,
            )
            if face_ref_image is not None:
                pipe_kwargs["ip_adapter_image"] = face_ref_image

            image = _generate_with_oom_retry(pipe, queue, pipe_kwargs, t_ref=t_ref)
            elapsed = time.time() - t_ref["t0"]

            name = _timestamp_name("gen")
            out_path = OUTPUTS_DIR / name
            image.save(out_path)
            print(f"[serveur] Image générée en {elapsed:.1f}s → {out_path}", flush=True)

            emit("done",
                 image_url=f"/outputs/{name}",
                 seed=used_seed,
                 elapsed_seconds=round(elapsed, 1),
                 face_lock=face_ref_image is not None,
                 mode=mode,
                 prompt=positive)
        except Exception as e:
            traceback.print_exc()
            emit("error", message=f"{type(e).__name__}: {e}")

    Thread(target=run, daemon=True).start()
    return StreamingResponse(_drain_queue_to_sse(queue), media_type="text/event-stream")


@app.post("/api/generate-character-stream")
def api_generate_character_stream(
    profile_path: str = Form(...),
    reroll: bool = Form(False),
    seed: Optional[int] = Form(None),
    width: int = Form(896),
    height: int = Form(1152),
    steps: int = Form(28),
    cfg: float = Form(7.0),
    mode: str = Form("standard"),
):
    queue: Queue = Queue()

    if mode == "fast":
        steps = 4
        cfg = 0.0
        width = min(width, 768)
        height = min(height, 768)

    def emit(event_type, **data):
        queue.put((event_type, data))

    def run():
        try:
            path = ROOT / profile_path
            if not path.exists():
                emit("error", message=f"Profil introuvable : {profile_path}")
                return

            character = Character.from_json_file(path)

            if seed is not None:
                used_seed = seed
            elif reroll or character.seed is None:
                used_seed = random.randint(0, 2**31 - 1)
            else:
                used_seed = character.seed

            need_load = _pipe_text2img is None or _pipe_text2img_mode != mode
            if need_load:
                if mode == "fast":
                    emit("status", message="Chargement de SDXL Turbo (mode rapide)… "
                         "(premier lancement : téléchargement ~7 Go, peut durer 5-15 min)")
                else:
                    emit("status", message="Chargement de SDXL base… (premier lancement : "
                         "téléchargement ~7 Go, peut durer 5-15 min)")
            pipe = get_text2img_pipe(mode=mode)

            generator = _make_torch_generator(used_seed)

            emit("ready", message=f"Portrait de {character.name} en cours…", total=steps, mode=mode)

            t_ref = {"t0": time.time()}
            cb = _make_step_callback(queue, steps, t_ref)

            portrait_kwargs = dict(
                prompt=character.build_portrait_prompt(),
                negative_prompt=DEFAULT_NEGATIVE if mode != "fast" else None,
                width=width,
                height=height,
                num_inference_steps=steps,
                guidance_scale=cfg,
                generator=generator,
                callback_on_step_end=cb,
            )
            image = _generate_with_oom_retry(pipe, queue, portrait_kwargs, t_ref=t_ref)
            elapsed = time.time() - t_ref["t0"]

            out_name = f"{path.stem}.png"
            out_path = CHARACTERS_DIR / out_name
            image.save(out_path)

            if character.seed != used_seed or character.portrait_path != str(out_path):
                character.seed = used_seed
                character.portrait_path = str(out_path)
                character.to_json(path)

            emit("done",
                 image_url=f"/characters/{out_name}",
                 seed=used_seed,
                 elapsed_seconds=round(elapsed, 1),
                 portrait_path=str(out_path.relative_to(ROOT)))
        except Exception as e:
            traceback.print_exc()
            emit("error", message=f"{type(e).__name__}: {e}")

    Thread(target=run, daemon=True).start()
    return StreamingResponse(_drain_queue_to_sse(queue), media_type="text/event-stream")


@app.post("/api/tryon-stream")
async def api_tryon_stream(
    person: UploadFile = File(...),
    garment: UploadFile = File(...),
    mask: Optional[UploadFile] = File(None),
    auto_mask: str = Form("none"),
    prompt: str = Form(...),
    garment_scale: float = Form(0.85),
    strength: float = Form(0.95),
    steps: int = Form(28),
    cfg: float = Form(7.5),
    seed: Optional[int] = Form(None),
):
    # Les UploadFile doivent être lus dans la coroutine (avant le thread).
    person_bytes = await person.read()
    garment_bytes = await garment.read()
    mask_bytes = await mask.read() if mask is not None else None

    queue: Queue = Queue()

    def emit(event_type, **data):
        queue.put((event_type, data))

    def run():
        try:
            from PIL import Image, ImageDraw, ImageFilter
            from ip_adapter_helpers import open_reference_image

            upload_dir = OUTPUTS_DIR / "uploads"
            upload_dir.mkdir(parents=True, exist_ok=True)

            person_path = upload_dir / _timestamp_name("person")
            garment_path = upload_dir / _timestamp_name("garment", "jpg")
            person_path.write_bytes(person_bytes)
            garment_path.write_bytes(garment_bytes)

            person_image = Image.open(person_path).convert("RGB")
            w, h = person_image.size
            garment_image = open_reference_image(garment_path)

            if mask_bytes is not None:
                mask_path = upload_dir / _timestamp_name("mask")
                mask_path.write_bytes(mask_bytes)
                mask_image = Image.open(mask_path).convert("L")
                if mask_image.size != (w, h):
                    mask_image = mask_image.resize((w, h), Image.NEAREST)
            elif auto_mask in ("hip", "torso"):
                mask_image = Image.new("L", (w, h), 0)
                draw = ImageDraw.Draw(mask_image)
                if auto_mask == "hip":
                    x1, y1, x2, y2 = int(0.20 * w), int(0.50 * h), int(0.80 * w), int(0.78 * h)
                else:
                    x1, y1, x2, y2 = int(0.18 * w), int(0.18 * h), int(0.82 * w), int(0.55 * h)
                draw.rectangle([x1, y1, x2, y2], fill=255)
                mask_image = mask_image.filter(ImageFilter.GaussianBlur(radius=10))
            else:
                emit("error", message="Fournir un masque OU auto_mask=hip|torso")
                return

            need_load = _pipe_inpaint is None
            if need_load:
                emit("status", message="Chargement SDXL Inpainting… (premier lancement : "
                     "téléchargement ~7 Go, peut durer 5-15 min)")
            if not _general_adapter_loaded_on_inpaint:
                emit("status", message="Chargement IP-Adapter général…")
            pipe = get_inpaint_pipe_with_general_adapter(scale=garment_scale)

            used_seed = seed if seed is not None else random.randint(0, 2**31 - 1)
            generator = _make_torch_generator(used_seed)

            full_prompt = (
                f"{prompt}, high detail fabric texture, realistic clothing, "
                "natural skin tone, seamless integration, photorealistic, 8k, sharp focus"
            )

            emit("ready", message="Essayage en cours…", total=steps)

            t_ref = {"t0": time.time()}
            cb = _make_step_callback(queue, steps, t_ref)

            tryon_kwargs = dict(
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
                callback_on_step_end=cb,
            )
            # Pour le try-on, le retry OOM ne peut pas réduire arbitrairement
            # la résolution (elle est dictée par la person_image). On laisse
            # le helper essayer mais en pratique il faut que l'utilisateur
            # uploade une image plus petite si OOM.
            image = _generate_with_oom_retry(pipe, queue, tryon_kwargs, t_ref=t_ref,
                                              fallback_size=min(w, h, 512))
            elapsed = time.time() - t_ref["t0"]

            name = _timestamp_name("tryon")
            out_path = OUTPUTS_DIR / name
            image.save(out_path)

            emit("done",
                 image_url=f"/outputs/{name}",
                 seed=used_seed,
                 elapsed_seconds=round(elapsed, 1))
        except Exception as e:
            traceback.print_exc()
            emit("error", message=f"{type(e).__name__}: {e}")

    Thread(target=run, daemon=True).start()
    return StreamingResponse(_drain_queue_to_sse(queue), media_type="text/event-stream")
