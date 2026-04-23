#!/usr/bin/env python3
"""Générateur d'images Lelins — script CLI.

Exécution locale d'un modèle Stable Diffusion XL pour produire des visuels de
catalogue. Détection automatique du matériel (MPS sur Mac, CUDA sur Windows
NVIDIA, CPU en secours).

Usage rapide :

    python scripts/generate.py --prompt "male model wearing black boxer briefs" \
        --output outputs/test.png

Usage avec le template produit :

    python scripts/generate.py --garment "navy blue trunks" \
        --model-style athletic --background studio_grey \
        --output outputs/navy.png
"""

from __future__ import annotations

import argparse
import sys
import time
from pathlib import Path

# Le dossier ``scripts`` doit être dans le path pour importer ``prompts``.
SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

from prompts import ProductPrompt, simple_prompt  # noqa: E402


DEFAULT_MODEL_ID = "stabilityai/stable-diffusion-xl-base-1.0"


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
    from diffusers import StableDiffusionXLPipeline

    pipe = StableDiffusionXLPipeline.from_pretrained(
        model_id,
        torch_dtype=dtype,
        use_safetensors=True,
        variant="fp16" if dtype.__repr__() == "torch.float16" else None,
    )
    pipe = pipe.to(device)

    if device == "mps":
        pipe.enable_attention_slicing()
    elif device == "cuda":
        try:
            pipe.enable_xformers_memory_efficient_attention()
        except Exception:
            pipe.enable_attention_slicing()
    else:
        pipe.enable_attention_slicing()

    pipe.set_progress_bar_config(disable=False)
    return pipe


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Générateur d'images Lelins (SDXL local)")

    p.add_argument("--check", action="store_true",
                   help="Affiche juste le matériel détecté et quitte")

    # Deux façons de fournir le prompt : libre ou structuré via template.
    p.add_argument("--prompt", type=str,
                   help="Prompt positif complet (mode libre)")
    p.add_argument("--negative", type=str, default=None,
                   help="Prompt négatif (défaut : négatif du template)")

    p.add_argument("--garment", type=str,
                   help="Type de vêtement, ex. 'black cotton boxer briefs' (mode template)")
    p.add_argument("--model-style", type=str, default="athletic",
                   choices=["athletic", "slim", "mature", "casual"],
                   help="Style du mannequin (mode template)")
    p.add_argument("--background", type=str, default="studio_white",
                   choices=["studio_white", "studio_grey", "studio_beige",
                            "loft", "outdoor_beach", "urban"],
                   help="Arrière-plan (mode template)")
    p.add_argument("--pose", type=str,
                   default="standing front view, arms relaxed at sides",
                   help="Pose du mannequin (mode template)")

    p.add_argument("--width", type=int, default=1024)
    p.add_argument("--height", type=int, default=1024)
    p.add_argument("--steps", type=int, default=30,
                   help="Nombre de pas de diffusion (20-40 typique)")
    p.add_argument("--cfg", type=float, default=7.0,
                   help="Classifier-free guidance scale (5-9 typique)")
    p.add_argument("--seed", type=int, default=None,
                   help="Seed pour reproduire un visuel (défaut : aléatoire)")

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

    # Résolution du prompt : mode libre OU mode template.
    if args.prompt:
        positive = args.prompt
        negative = args.negative or simple_prompt(garment="placeholder")[1]
    elif args.garment:
        positive, negative_default = simple_prompt(
            garment=args.garment,
            model_style=args.model_style,
            background=args.background,
            pose=args.pose,
        )
        negative = args.negative or negative_default
    else:
        print("ERREUR : fournir --prompt ou --garment.", file=sys.stderr)
        return 2

    print("\n--- Prompt positif ---")
    print(positive)
    print("\n--- Prompt négatif ---")
    print(negative)
    print()

    # Seed déterministe si demandée.
    generator = None
    if args.seed is not None:
        generator = torch.Generator(device=device if device != "mps" else "cpu")
        generator = generator.manual_seed(args.seed)

    print(f"Chargement du modèle '{args.model_id}'...")
    t0 = time.time()
    pipe = build_pipeline(args.model_id, device, dtype)
    print(f"Modèle prêt en {time.time() - t0:.1f}s.")

    print("Génération en cours...")
    t0 = time.time()
    result = pipe(
        prompt=positive,
        negative_prompt=negative,
        width=args.width,
        height=args.height,
        num_inference_steps=args.steps,
        guidance_scale=args.cfg,
        generator=generator,
    )
    image = result.images[0]
    print(f"Image générée en {time.time() - t0:.1f}s.")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    image.save(args.output)
    print(f"Image sauvegardée : {args.output}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
