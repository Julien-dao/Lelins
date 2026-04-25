#!/usr/bin/env python3
"""Génération d'un portrait de mannequin (Character) pour Lelins.

Lit un profil JSON décrivant le mannequin (40 paramètres : peau, cheveux, yeux,
visage, corps, tatouages…), produit un portrait studio photoréaliste et
sauvegarde la seed dans le profil pour reproductibilité ultérieure.

Usage :

    # 1. Première génération (la seed sera tirée et écrite dans le JSON)
    python scripts/generate_character.py \\
        --profile characters/lelins-main.json \\
        --output characters/lelins-main.png

    # 2. Re-génération à l'identique (la seed est lue dans le JSON)
    python scripts/generate_character.py \\
        --profile characters/lelins-main.json \\
        --output characters/lelins-main-v2.png

    # 3. Forcer une nouvelle seed pour explorer
    python scripts/generate_character.py \\
        --profile characters/lelins-main.json --reroll \\
        --output characters/lelins-main-alt.png

Astuce : pour ajuster un trait (ex. couleur d'yeux), éditer le JSON et relancer
en gardant la même seed → la modification se voit clairement, le reste change peu.
"""

from __future__ import annotations

import argparse
import random
import sys
import time
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

from generate import DEFAULT_MODEL_ID, build_pipeline, detect_device  # noqa: E402
from prompts import DEFAULT_NEGATIVE, Character  # noqa: E402


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Génération d'un portrait Character (Lelins)")
    p.add_argument("--profile", "-p", type=Path, required=True,
                   help="Fichier JSON du profil mannequin")
    p.add_argument("--output", "-o", type=Path, required=True,
                   help="Chemin de sortie de l'image (PNG)")
    p.add_argument("--reroll", action="store_true",
                   help="Ignorer la seed enregistrée et en tirer une nouvelle")
    p.add_argument("--seed", type=int, default=None,
                   help="Forcer une seed précise (prioritaire sur --reroll)")

    p.add_argument("--width", type=int, default=896,
                   help="Largeur en pixels (portrait : 896 par défaut)")
    p.add_argument("--height", type=int, default=1152,
                   help="Hauteur en pixels (portrait : 1152 par défaut)")
    p.add_argument("--steps", type=int, default=35)
    p.add_argument("--cfg", type=float, default=7.0)
    p.add_argument("--model-id", type=str, default=DEFAULT_MODEL_ID)

    p.add_argument("--negative", type=str, default=None,
                   help="Prompt négatif additionnel (concaténé au défaut)")
    return p.parse_args()


def main() -> int:
    args = parse_args()

    try:
        import torch
    except ImportError:
        print("ERREUR : PyTorch n'est pas installé. Voir docs/INSTALL.md.", file=sys.stderr)
        return 1

    if not args.profile.exists():
        print(f"ERREUR : profil introuvable : {args.profile}", file=sys.stderr)
        return 2

    character = Character.from_json_file(args.profile)
    print(f"Mannequin : {character.name}")

    # Résolution de la seed : priorité --seed > --reroll > seed du JSON > aléatoire.
    if args.seed is not None:
        seed = args.seed
        print(f"Seed forcée par --seed : {seed}")
    elif args.reroll or character.seed is None:
        seed = random.randint(0, 2**31 - 1)
        print(f"Nouvelle seed tirée : {seed}")
    else:
        seed = character.seed
        print(f"Seed du profil réutilisée : {seed}")

    positive = character.build_portrait_prompt()
    negative = DEFAULT_NEGATIVE
    if args.negative:
        negative = f"{negative}, {args.negative}"

    print("\n--- Prompt positif ---")
    print(positive)
    print("\n--- Prompt négatif ---")
    print(negative)
    print()

    device, dtype = detect_device()
    print(f"Matériel détecté : device={device}, dtype={dtype}")
    print(f"Chargement du modèle '{args.model_id}'...")
    t0 = time.time()
    pipe = build_pipeline(args.model_id, device, dtype)
    print(f"Modèle prêt en {time.time() - t0:.1f}s.")

    generator = torch.Generator(device=device if device != "mps" else "cpu")
    generator = generator.manual_seed(seed)

    print("Génération du portrait...")
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
    print(f"Portrait généré en {time.time() - t0:.1f}s.")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    image.save(args.output)
    print(f"Image sauvegardée : {args.output}")

    # On enregistre la seed et le chemin du portrait dans le profil pour la
    # cohérence future (IP-Adapter face s'en sert comme référence d'identité).
    portrait_path_str = str(args.output)
    needs_save = (character.seed != seed) or (character.portrait_path != portrait_path_str)
    if needs_save:
        character.seed = seed
        character.portrait_path = portrait_path_str
        character.to_json(args.profile)
        print(f"Profil mis à jour : seed={seed}, portrait_path={portrait_path_str}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
