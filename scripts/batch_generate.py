#!/usr/bin/env python3
"""Génération en lot depuis un fichier JSON — pour produire un catalogue
entier en une seule exécution.

Le JSON d'entrée est une liste d'objets, chacun décrivant un visuel. Le modèle
est chargé une seule fois, ce qui rend le lot largement plus rapide que des
appels individuels à ``generate.py``.

Format d'un item :

    {
      "name": "boxer-noir-coton",            # nom de fichier de sortie
      "garment": "black cotton boxer briefs", # champ principal du template
      "model_style": "athletic",              # athletic|slim|mature|casual
      "background": "studio_white",           # voir prompts/templates.py
      "pose": "standing front view, arms relaxed at sides",
      "seed": 1234,                           # optionnel, reproductibilité
      "variations": 3                         # optionnel, nb d'images (défaut 1)
    }

Alternative : fournir directement ``prompt`` / ``negative`` pour un contrôle
total au lieu du template.

Usage :

    python scripts/batch_generate.py --input scripts/prompts/examples.json \\
        --output-dir outputs/catalog
"""

from __future__ import annotations

import argparse
import json
import random
import sys
import time
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

from generate import DEFAULT_MODEL_ID, build_pipeline, detect_device  # noqa: E402
from prompts import simple_prompt  # noqa: E402


def resolve_prompts(item: dict) -> tuple[str, str]:
    """Retourne (positif, négatif) pour un item, en mode libre ou template."""
    if "prompt" in item:
        positive = item["prompt"]
        negative = item.get("negative") or simple_prompt(garment="placeholder")[1]
        return positive, negative

    if "garment" not in item:
        raise ValueError(
            f"Item invalide (ni 'prompt' ni 'garment') : {item.get('name', '?')}"
        )

    positive, negative_default = simple_prompt(
        garment=item["garment"],
        model_style=item.get("model_style", "athletic"),
        background=item.get("background", "studio_white"),
        pose=item.get("pose", "standing front view, arms relaxed at sides"),
    )
    negative = item.get("negative") or negative_default
    return positive, negative


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Génération d'images en lot (Lelins)")
    p.add_argument("--input", "-i", type=Path, required=True,
                   help="Fichier JSON décrivant les visuels à générer")
    p.add_argument("--output-dir", "-o", type=Path, default=Path("outputs/batch"),
                   help="Dossier de sortie")
    p.add_argument("--model-id", type=str, default=DEFAULT_MODEL_ID)
    p.add_argument("--width", type=int, default=1024)
    p.add_argument("--height", type=int, default=1024)
    p.add_argument("--steps", type=int, default=30)
    p.add_argument("--cfg", type=float, default=7.0)
    return p.parse_args()


def main() -> int:
    args = parse_args()

    try:
        import torch
    except ImportError:
        print("ERREUR : PyTorch n'est pas installé. Voir docs/INSTALL.md.", file=sys.stderr)
        return 1

    if not args.input.exists():
        print(f"ERREUR : fichier introuvable : {args.input}", file=sys.stderr)
        return 2

    items = json.loads(args.input.read_text(encoding="utf-8"))
    if not isinstance(items, list) or not items:
        print("ERREUR : le JSON doit être une liste non vide.", file=sys.stderr)
        return 2

    args.output_dir.mkdir(parents=True, exist_ok=True)

    device, dtype = detect_device()
    print(f"Matériel détecté : device={device}, dtype={dtype}")
    print(f"Chargement du modèle '{args.model_id}'...")
    t0 = time.time()
    pipe = build_pipeline(args.model_id, device, dtype)
    print(f"Modèle prêt en {time.time() - t0:.1f}s.")

    total_images = sum(max(1, int(item.get("variations", 1))) for item in items)
    print(f"\n{len(items)} item(s), {total_images} image(s) à générer au total.\n")

    counter = 0
    for idx, item in enumerate(items, start=1):
        name = item.get("name", f"item_{idx:03d}")
        variations = max(1, int(item.get("variations", 1)))

        try:
            positive, negative = resolve_prompts(item)
        except ValueError as e:
            print(f"[{idx}/{len(items)}] SKIP {name} : {e}")
            continue

        base_seed = item.get("seed")
        for v in range(variations):
            counter += 1
            seed = base_seed + v if isinstance(base_seed, int) else random.randint(0, 2**31 - 1)
            generator = torch.Generator(device=device if device != "mps" else "cpu")
            generator = generator.manual_seed(seed)

            suffix = f"_v{v + 1}" if variations > 1 else ""
            output_path = args.output_dir / f"{name}{suffix}.png"

            print(f"[{counter}/{total_images}] {output_path.name} (seed={seed})")
            t = time.time()
            result = pipe(
                prompt=positive,
                negative_prompt=negative,
                width=args.width,
                height=args.height,
                num_inference_steps=args.steps,
                guidance_scale=args.cfg,
                generator=generator,
            )
            result.images[0].save(output_path)
            print(f"           {time.time() - t:.1f}s -> {output_path}")

    print(f"\nTerminé. {counter} image(s) dans {args.output_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
