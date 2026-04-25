#!/usr/bin/env python3
"""Essayage virtuel : faire porter un vêtement réel à un mannequin généré.

Approche : **SDXL Inpainting + IP-Adapter** sur l'image du vêtement.

- L'utilisateur fournit :
    * --person  : image du mannequin (généralement issue de generate.py)
    * --garment : photo packshot du vêtement réel (sous-vêtement, t-shirt…)
    * --mask    : masque PNG (blanc = zone à re-générer = la zone vêtue)
    * --prompt  : texte décrivant le vêtement (couleur, matière)
- SDXL Inpainting redessine la zone masquée en respectant l'image de référence
  passée à IP-Adapter (le vêtement réel) → on essaie de reproduire son
  apparence (couleur, coupe, texture) sur le mannequin.

Limites honnêtes
----------------
- Le rendu **n'est pas pixel-perfect** sur les logos, textes, motifs très fins.
  Pour cela, voir docs/TRYON.md (option avancée : CatVTON / IDM-VTON).
- Le résultat dépend beaucoup du **masque** : trop large, le corps change ;
  trop étroit, on voit la jonction. Voir scripts/make_mask.py.
- Sur Mac MPS, ce pipeline peut être 1.5-2× plus lent que generate.py
  (l'inpainting + l'image encoder IP-Adapter cumulent).

Usage
-----

    python scripts/tryon.py \\
        --person outputs/main-base.png \\
        --garment photos/boxer-noir.jpg \\
        --mask masks/boxer-zone.png \\
        --prompt "black cotton boxer briefs with elastic waistband" \\
        --output outputs/tryon-boxer-noir.png
"""

from __future__ import annotations

import argparse
import sys
import time
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

from prompts import DEFAULT_NEGATIVE  # noqa: E402


DEFAULT_INPAINT_MODEL_ID = "diffusers/stable-diffusion-xl-1.0-inpainting-0.1"


def detect_device() -> tuple[str, "torch.dtype"]:
    """Retourne (device, dtype) adapté à la machine."""
    import torch

    if torch.cuda.is_available():
        return "cuda", torch.float16
    if getattr(torch.backends, "mps", None) and torch.backends.mps.is_available():
        return "mps", torch.float16
    return "cpu", torch.float32


def build_inpaint_pipeline(model_id: str, device: str, dtype):
    """Charge le pipeline SDXL Inpainting et applique les optimisations."""
    from diffusers import StableDiffusionXLInpaintPipeline

    pipe = StableDiffusionXLInpaintPipeline.from_pretrained(
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
    p = argparse.ArgumentParser(description="Essayage virtuel SDXL Inpainting + IP-Adapter")

    p.add_argument("--person", "-p", type=Path, required=True,
                   help="Image du mannequin sur laquelle appliquer le vêtement")
    p.add_argument("--garment", "-g", type=Path, required=True,
                   help="Image du vêtement réel (photo packshot)")
    p.add_argument("--mask", "-m", type=Path, required=True,
                   help="Masque PNG (blanc = zone à re-générer)")

    p.add_argument("--prompt", type=str, required=True,
                   help="Description textuelle du vêtement (couleur, matière, coupe)")
    p.add_argument("--negative", type=str, default=None,
                   help="Prompt négatif additionnel (concaténé au défaut)")

    p.add_argument("--garment-scale", type=float, default=0.85,
                   help="Force du transfert d'apparence du vêtement (0.5-1.0, défaut 0.85)")
    p.add_argument("--strength", type=float, default=0.95,
                   help="Force d'inpainting (0.7-1.0, défaut 0.95). "
                        "1.0 = redessine entièrement, <1 conserve une partie.")

    p.add_argument("--steps", type=int, default=35)
    p.add_argument("--cfg", type=float, default=7.5)
    p.add_argument("--seed", type=int, default=None,
                   help="Seed reproductible (défaut : aléatoire)")

    p.add_argument("--model-id", type=str, default=DEFAULT_INPAINT_MODEL_ID)
    p.add_argument("--output", "-o", type=Path, default=Path("outputs/tryon.png"),
                   help="Chemin de sortie")
    return p.parse_args()


def main() -> int:
    args = parse_args()

    try:
        import torch
    except ImportError:
        print("ERREUR : PyTorch n'est pas installé. Voir docs/INSTALL.md.", file=sys.stderr)
        return 1

    for path in [args.person, args.garment, args.mask]:
        if not path.exists():
            print(f"ERREUR : fichier introuvable : {path}", file=sys.stderr)
            return 2

    from PIL import Image
    from ip_adapter_helpers import load_general_adapter, open_reference_image

    device, dtype = detect_device()
    print(f"Matériel détecté : device={device}, dtype={dtype}")

    person_image = Image.open(args.person).convert("RGB")
    mask_image = Image.open(args.mask).convert("L")  # niveaux de gris
    garment_image = open_reference_image(args.garment)

    # Le masque doit avoir la même taille que la person.
    if mask_image.size != person_image.size:
        print(f"Redimensionnement masque {mask_image.size} -> {person_image.size}")
        mask_image = mask_image.resize(person_image.size, Image.NEAREST)

    width, height = person_image.size
    print(f"Image personne : {width}x{height}")

    print(f"Chargement du modèle inpainting '{args.model_id}'...")
    t0 = time.time()
    pipe = build_inpaint_pipeline(args.model_id, device, dtype)
    print(f"Modèle prêt en {time.time() - t0:.1f}s.")

    print(f"Chargement IP-Adapter général (scale={args.garment_scale})...")
    load_general_adapter(pipe, scale=args.garment_scale)

    negative = DEFAULT_NEGATIVE
    if args.negative:
        negative = f"{negative}, {args.negative}"

    # Boost qualité spécifique try-on : on pousse le rendu textile.
    full_prompt = (
        f"{args.prompt}, high detail fabric texture, realistic clothing, "
        "natural skin tone, seamless integration, photorealistic, 8k, sharp focus"
    )

    print("\n--- Prompt positif ---")
    print(full_prompt)
    print("\n--- Prompt négatif ---")
    print(negative)
    print()

    generator = None
    if args.seed is not None:
        generator = torch.Generator(device=device if device != "mps" else "cpu")
        generator = generator.manual_seed(args.seed)
        print(f"Seed : {args.seed}")

    print("Inpainting en cours...")
    t0 = time.time()
    result = pipe(
        prompt=full_prompt,
        negative_prompt=negative,
        image=person_image,
        mask_image=mask_image,
        ip_adapter_image=garment_image,
        width=width,
        height=height,
        num_inference_steps=args.steps,
        guidance_scale=args.cfg,
        strength=args.strength,
        generator=generator,
    )
    image = result.images[0]
    print(f"Inpainting terminé en {time.time() - t0:.1f}s.")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    image.save(args.output)
    print(f"Image sauvegardée : {args.output}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
