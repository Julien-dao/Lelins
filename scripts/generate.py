#!/usr/bin/env python3
"""Générateur d'images Lelins — script CLI.

Exécution locale d'un modèle Stable Diffusion XL pour produire des visuels de
catalogue. Détection automatique du matériel (MPS sur Mac, CUDA sur Windows
NVIDIA, CPU en secours).

Trois modes d'usage :

1. Mode libre (prompt brut) :

    python scripts/generate.py --prompt "male model wearing black boxer briefs" \\
        --output outputs/test.png

2. Mode preset simple (modèle de mannequin générique) :

    python scripts/generate.py --garment "navy blue trunks" \\
        --model-style athletic --background studio_grey \\
        --output outputs/navy.png

3. Mode mannequin récurrent (Character JSON, 40 paramètres détaillés) :

    python scripts/generate.py --character characters/lelins-main.json \\
        --garment "black cotton boxer briefs" --background studio_white \\
        --output outputs/main-boxer-noir.png

Quand un Character est fourni, sa seed enregistrée est réutilisée par défaut
pour préserver l'identité visuelle du mannequin entre les visuels.
"""

from __future__ import annotations

import argparse
import sys
import time
from pathlib import Path

# Le dossier ``scripts`` doit être dans le path pour importer ``prompts``.
SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

from prompts import Character, ProductPrompt, simple_prompt  # noqa: E402


DEFAULT_MODEL_ID = "stabilityai/stable-diffusion-xl-base-1.0"
SDXL_TURBO_MODEL_ID = "stabilityai/sdxl-turbo"

# VAE corrigé pour éviter l'overflow fp16 sur Mac MPS qui produit des
# images entièrement noires. Voir https://huggingface.co/madebyollin/sdxl-vae-fp16-fix
SDXL_VAE_FP16_FIX = "madebyollin/sdxl-vae-fp16-fix"


def detect_device() -> tuple[str, "torch.dtype"]:
    """Retourne (device, dtype) adapté à la machine."""
    import torch

    if torch.cuda.is_available():
        return "cuda", torch.float16
    if getattr(torch.backends, "mps", None) and torch.backends.mps.is_available():
        return "mps", torch.float16
    return "cpu", torch.float32


def build_pipeline(model_id: str, device: str, dtype):
    """Charge le pipeline SDXL et applique les optimisations adaptées."""
    from diffusers import AutoencoderKL, StableDiffusionXLPipeline

    is_fp16 = dtype.__repr__() == "torch.float16"

    kwargs = dict(
        torch_dtype=dtype,
        use_safetensors=True,
        variant="fp16" if is_fp16 else None,
    )

    # Sur Mac MPS en fp16, le VAE par défaut souffre d'un overflow numérique
    # qui produit des images noires. On le remplace par le VAE corrigé.
    if device == "mps" and is_fp16:
        kwargs["vae"] = AutoencoderKL.from_pretrained(
            SDXL_VAE_FP16_FIX, torch_dtype=dtype,
        )

    pipe = StableDiffusionXLPipeline.from_pretrained(model_id, **kwargs)
    pipe = pipe.to(device)

    if device == "mps":
        # Optimisations mémoire critiques sur Apple Silicon, surtout 8 Go.
        pipe.enable_attention_slicing()
        pipe.enable_vae_slicing()
        pipe.enable_vae_tiling()
    elif device == "cuda":
        try:
            pipe.enable_xformers_memory_efficient_attention()
        except Exception:
            pipe.enable_attention_slicing()
    else:
        pipe.enable_attention_slicing()
        pipe.enable_vae_slicing()
        pipe.enable_vae_tiling()

    pipe.set_progress_bar_config(disable=False)
    return pipe


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Générateur d'images Lelins (SDXL local)")

    p.add_argument("--check", action="store_true",
                   help="Affiche juste le matériel détecté et quitte")

    # Mode libre.
    p.add_argument("--prompt", type=str,
                   help="Prompt positif complet (mode libre)")
    p.add_argument("--negative", type=str, default=None,
                   help="Prompt négatif (défaut : négatif du template)")

    # Mode template (preset simple OU character détaillé).
    p.add_argument("--character", "-c", type=Path, default=None,
                   help="Profil mannequin JSON (40 paramètres). Prioritaire sur --model-style.")
    p.add_argument("--garment", type=str,
                   help="Type de vêtement, ex. 'black cotton boxer briefs'")
    p.add_argument("--model-style", type=str, default="athletic",
                   choices=["athletic", "slim", "mature", "casual"],
                   help="Preset simple (utilisé si --character n'est pas fourni)")
    p.add_argument("--background", type=str, default="studio_white",
                   choices=["studio_white", "studio_grey", "studio_beige",
                            "loft", "outdoor_beach", "urban"],
                   help="Arrière-plan")
    p.add_argument("--pose", type=str,
                   default="standing front view, arms relaxed at sides",
                   help="Pose du mannequin")

    # Paramètres de diffusion.
    p.add_argument("--width", type=int, default=1024)
    p.add_argument("--height", type=int, default=1024)
    p.add_argument("--steps", type=int, default=30,
                   help="Nombre de pas de diffusion (20-40 typique)")
    p.add_argument("--cfg", type=float, default=7.0,
                   help="Classifier-free guidance scale (5-9 typique)")
    p.add_argument("--seed", type=int, default=None,
                   help="Seed pour reproduire un visuel (défaut : seed du Character "
                        "si fourni, sinon aléatoire)")

    # Verrouillage du visage (IP-Adapter face).
    p.add_argument("--face-reference", type=Path, default=None,
                   help="Image de référence pour verrouiller le visage via IP-Adapter. "
                        "Si --character est fourni et qu'il a un portrait_path, "
                        "celui-ci est utilisé automatiquement.")
    p.add_argument("--face-scale", type=float, default=0.7,
                   help="Force du verrouillage facial (0.5-1.0, défaut 0.7)")
    p.add_argument("--no-face-lock", action="store_true",
                   help="Désactive l'IP-Adapter face même si un portrait est dispo")

    p.add_argument("--model-id", type=str, default=DEFAULT_MODEL_ID,
                   help=f"ID HuggingFace du modèle (défaut : {DEFAULT_MODEL_ID})")
    p.add_argument("--output", "-o", type=Path, default=Path("outputs/out.png"),
                   help="Chemin de sortie de l'image")

    return p.parse_args()


def main() -> int:
    args = parse_args()

    try:
        import torch
    except ImportError:
        print("ERREUR : PyTorch n'est pas installé. Voir docs/INSTALL.md.", file=sys.stderr)
        return 1

    device, dtype = detect_device()
    print(f"Matériel détecté : device={device}, dtype={dtype}")

    if args.check:
        print("OK — environnement prêt. Aucun modèle téléchargé.")
        return 0

    # Résolution du Character éventuel.
    character: Character | None = None
    if args.character is not None:
        if not args.character.exists():
            print(f"ERREUR : Character introuvable : {args.character}", file=sys.stderr)
            return 2
        character = Character.from_json_file(args.character)
        print(f"Character chargé : {character.name}")

    # Résolution du prompt.
    if args.prompt:
        positive = args.prompt
        negative = args.negative or simple_prompt(garment="placeholder")[1]
    elif args.garment:
        positive, negative_default = ProductPrompt(
            garment=args.garment,
            character=character,
            model_style=args.model_style,
            background=args.background,
            pose=args.pose,
        ).build()
        negative = args.negative or negative_default
    else:
        print("ERREUR : fournir --prompt ou --garment.", file=sys.stderr)
        return 2

    print("\n--- Prompt positif ---")
    print(positive)
    print("\n--- Prompt négatif ---")
    print(negative)
    print()

    # Résolution de la seed : --seed > seed du Character > aléatoire.
    seed = args.seed
    if seed is None and character is not None and character.seed is not None:
        seed = character.seed
        print(f"Seed du Character réutilisée : {seed}")

    generator = None
    if seed is not None:
        generator = torch.Generator(device=device if device != "mps" else "cpu")
        generator = generator.manual_seed(seed)

    print(f"Chargement du modèle '{args.model_id}'...")
    t0 = time.time()
    pipe = build_pipeline(args.model_id, device, dtype)
    print(f"Modèle prêt en {time.time() - t0:.1f}s.")

    # Détermination de la référence visage pour l'IP-Adapter.
    face_ref_path: Path | None = None
    if not args.no_face_lock:
        if args.face_reference is not None:
            face_ref_path = args.face_reference
        elif character is not None and character.portrait_path:
            candidate = Path(character.portrait_path)
            if not candidate.is_absolute():
                # On résout par rapport au CWD, c'est ce que generate_character.py écrit.
                candidate = Path.cwd() / candidate
            if candidate.exists():
                face_ref_path = candidate

    face_ref_image = None
    if face_ref_path is not None:
        if not face_ref_path.exists():
            print(f"ATTENTION : référence visage introuvable : {face_ref_path}", file=sys.stderr)
        else:
            from ip_adapter_helpers import load_face_adapter, open_reference_image
            print(f"Chargement IP-Adapter face (scale={args.face_scale})...")
            load_face_adapter(pipe, scale=args.face_scale)
            face_ref_image = open_reference_image(face_ref_path)
            print(f"Référence visage : {face_ref_path}")

    print("Génération en cours...")
    t0 = time.time()
    pipe_kwargs = dict(
        prompt=positive,
        negative_prompt=negative,
        width=args.width,
        height=args.height,
        num_inference_steps=args.steps,
        guidance_scale=args.cfg,
        generator=generator,
    )
    if face_ref_image is not None:
        pipe_kwargs["ip_adapter_image"] = face_ref_image

    result = pipe(**pipe_kwargs)
    image = result.images[0]
    print(f"Image générée en {time.time() - t0:.1f}s.")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    image.save(args.output)
    print(f"Image sauvegardée : {args.output}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
