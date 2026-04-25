#!/usr/bin/env python3
"""Helper de création de masque pour ``tryon.py``.

Le masque définit la zone à re-générer (blanc) vs. à conserver (noir).
Trois modes au choix :

1. **--rect** : rectangle explicite (coordonnées en pixels ou en pourcentage).
2. **--auto-hip** : heuristique pour cibler la zone hanches/sous-vêtements
   (rectangle centré, environ 35-65 % de la hauteur, 20-80 % de la largeur).
3. **--auto-torso** : heuristique pour cibler le torse (15-55 % de la hauteur).

Pour un masque précis (ex. logo, motif), on peut aussi le **dessiner à la main**
dans un éditeur d'image (Photoshop, GIMP, Photopea…) puis le passer directement
à ``tryon.py --mask``.

Usage :

    # Auto pour zone sous-vêtement, à partir d'une photo person
    python scripts/make_mask.py --person outputs/main-base.png \\
        --auto-hip --output masks/boxer-zone.png

    # Rectangle explicite (en pixels)
    python scripts/make_mask.py --person outputs/main-base.png \\
        --rect 200,500,800,900 --output masks/zone.png

    # Rectangle en pourcentage de la taille de l'image (utile cross-résolution)
    python scripts/make_mask.py --person outputs/main-base.png \\
        --rect-pct 0.2,0.5,0.8,0.9 --output masks/zone.png
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


def parse_rect(s: str, w: int, h: int, percent: bool) -> tuple[int, int, int, int]:
    parts = [p.strip() for p in s.split(",")]
    if len(parts) != 4:
        raise ValueError("Format attendu : x1,y1,x2,y2")
    if percent:
        x1, y1, x2, y2 = (float(p) for p in parts)
        return (int(x1 * w), int(y1 * h), int(x2 * w), int(y2 * h))
    return tuple(int(p) for p in parts)  # type: ignore[return-value]


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Créateur de masque pour tryon.py")
    p.add_argument("--person", type=Path, required=True,
                   help="Image dont on veut le masque (sert à connaître la résolution)")
    p.add_argument("--output", "-o", type=Path, required=True,
                   help="Chemin de sortie du masque PNG")

    mode = p.add_mutually_exclusive_group(required=True)
    mode.add_argument("--rect", type=str,
                      help="Rectangle en pixels : x1,y1,x2,y2")
    mode.add_argument("--rect-pct", type=str,
                      help="Rectangle en pourcentage : x1,y1,x2,y2 dans [0,1]")
    mode.add_argument("--auto-hip", action="store_true",
                      help="Heuristique zone sous-vêtements : ~35-65% hauteur")
    mode.add_argument("--auto-torso", action="store_true",
                      help="Heuristique zone torse : ~15-55% hauteur")

    p.add_argument("--feather", type=int, default=10,
                   help="Adoucissement des bords du masque en pixels (défaut 10)")
    return p.parse_args()


def main() -> int:
    args = parse_args()

    try:
        from PIL import Image, ImageDraw, ImageFilter
    except ImportError:
        print("ERREUR : Pillow non installé.", file=sys.stderr)
        return 1

    if not args.person.exists():
        print(f"ERREUR : image introuvable : {args.person}", file=sys.stderr)
        return 2

    person = Image.open(args.person)
    w, h = person.size

    if args.rect:
        x1, y1, x2, y2 = parse_rect(args.rect, w, h, percent=False)
    elif args.rect_pct:
        x1, y1, x2, y2 = parse_rect(args.rect_pct, w, h, percent=True)
    elif args.auto_hip:
        x1, y1, x2, y2 = int(0.20 * w), int(0.50 * h), int(0.80 * w), int(0.78 * h)
    elif args.auto_torso:
        x1, y1, x2, y2 = int(0.18 * w), int(0.18 * h), int(0.82 * w), int(0.55 * h)
    else:
        print("ERREUR : aucun mode sélectionné.", file=sys.stderr)
        return 2

    mask = Image.new("L", (w, h), 0)
    draw = ImageDraw.Draw(mask)
    draw.rectangle([x1, y1, x2, y2], fill=255)

    if args.feather > 0:
        mask = mask.filter(ImageFilter.GaussianBlur(radius=args.feather))

    args.output.parent.mkdir(parents=True, exist_ok=True)
    mask.save(args.output)
    print(f"Masque {w}x{h} (rect={x1},{y1},{x2},{y2}, feather={args.feather}px) "
          f"sauvegardé : {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
