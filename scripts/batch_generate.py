#!/usr/bin/env python3
"""Génération en lot depuis un fichier JSON — pour produire un catalogue
entier en une seule exécution.

Le JSON d'entrée est une liste d'objets, chacun décrivant un visuel. Le modèle
est chargé une seule fois, ce qui rend le lot largement plus rapide que des
appels individuels à ``generate.py``.

Format d'un item (mode template) :

    {
      "name": "boxer-noir-coton",
      "garment": "black cotton boxer briefs",
      "character": "characters/lelins-main.json",  # OPTION A : chemin vers un Character
      "model_style": "athletic",                   # OPTION B : preset simple si pas de Character
      "background": "studio_white",
      "pose": "standing front view, arms relaxed at sides",
      "seed": 1234,                                # optionnel : prioritaire sur la seed du Character
      "variations": 3
    }

Alternative complète : fournir ``prompt`` / ``negative`` au lieu du template.

Si un ``character`` est défini et qu'aucune ``seed`` n'est précisée, on
réutilise la seed enregistrée dans le profil — l'identité du mannequin est
préservée à travers tout le catalogue.

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
from prompts import Character, ProductPrompt, simple_prompt  # noqa: E402


# Cache des Characters chargés depuis disque (évite de relire le JSON à chaque variation).
_CHARACTER_CACHE: dict[str, Character] = {}


def _load_character(path_str: str, base: Path) -> Character:
    """Charge un Character depuis le chemin (relatif au CWD ou au fichier d'input)."""
    candidates = [Path(path_str), base.parent / path_str, base / path_str]
    for c in candidates:
        if c.exists():
            cache_key = str(c.resolve())
            if cache_key not in _CHARACTER_CACHE:
                _CHARACTER_CACHE[cache_key] = Character.from_json_file(c)
            return _CHARACTER_CACHE[cache_key]
    raise FileNotFoundError(f"Character introuvable : {path_str}")


def resolve_prompts_and_seed(
    item: dict,
    input_dir: Path,
) -> tuple[str, str, int | None, Character | None]:
    """Retourne (positif, négatif, seed_par_défaut_du_character_ou_None, character_ou_None)."""
    if "prompt" in item:
        positive = item["prompt"]
        negative = item.get("negative") or simple_prompt(garment="placeholder")[1]
        return positive, negative, None, None

    if "garment" not in item:
        raise ValueError(
            f"Item invalide (ni 'prompt' ni 'garment') : {item.get('name', '?')}"
        )

    character = None
    char_seed = None
    if "character" in item and item["character"]:
        character = _load_character(item["character"], input_dir)
        char_seed = character.seed

    positive, negative_default = ProductPrompt(
        garment=item["garment"],
        character=character,
        model_style=item.get("model_style", "athletic"),
        background=item.get("background", "studio_white"),
        pose=item.get("pose", "standing front view, arms relaxed at sides"),
    ).build()
    negative = item.get("negative") or negative_default
    return positive, negative, char_seed, character


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
    p.add_argument("--face-scale", type=float, default=0.7,
                   help="Force du verrouillage facial IP-Adapter (0.5-1.0, défaut 0.7)")
    p.add_argument("--no-face-lock", action="store_true",
                   help="Désactive l'IP-Adapter face même si des portraits sont dispos")
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

    # Pré-résolution des items pour décider du chargement IP-Adapter face.
    resolved: list[tuple[int, dict, str, str, int | None, Character | None]] = []
    for idx, item in enumerate(items, start=1):
        try:
            positive, negative, character_seed, character = resolve_prompts_and_seed(item, args.input)
            resolved.append((idx, item, positive, negative, character_seed, character))
        except (ValueError, FileNotFoundError) as e:
            print(f"[{idx}/{len(items)}] SKIP {item.get('name', '?')} : {e}")

    # Si au moins un Character a un portrait, on charge l'IP-Adapter face.
    use_face_lock = (not args.no_face_lock) and any(
        c is not None and c.portrait_path and Path(c.portrait_path).exists()
        for _, _, _, _, _, c in resolved
    )
    face_image_cache: dict[str, "Image.Image"] = {}
    if use_face_lock:
        from ip_adapter_helpers import load_face_adapter, open_reference_image
        from PIL import Image  # noqa: F401  (pour le type hint dans le cache)
        print(f"Chargement IP-Adapter face (scale={args.face_scale})...")
        load_face_adapter(pipe, scale=args.face_scale)

    total_images = sum(max(1, int(item.get("variations", 1))) for _, item, *_ in resolved)
    print(f"\n{len(resolved)} item(s), {total_images} image(s) à générer au total.\n")

    counter = 0
    for idx, item, positive, negative, character_seed, character in resolved:
        name = item.get("name", f"item_{idx:03d}")
        variations = max(1, int(item.get("variations", 1)))

        # Image de référence visage pour cet item.
        face_ref_image = None
        if use_face_lock and character is not None and character.portrait_path:
            p = Path(character.portrait_path)
            if p.exists():
                if str(p) not in face_image_cache:
                    face_image_cache[str(p)] = open_reference_image(p)
                face_ref_image = face_image_cache[str(p)]

        # Priorité de la seed : item["seed"] > seed du Character > aléatoire.
        base_seed = item.get("seed") if item.get("seed") is not None else character_seed
        for v in range(variations):
            counter += 1
            seed = base_seed + v if isinstance(base_seed, int) else random.randint(0, 2**31 - 1)
            generator = torch.Generator(device=device if device != "mps" else "cpu")
            generator = generator.manual_seed(seed)

            suffix = f"_v{v + 1}" if variations > 1 else ""
            output_path = args.output_dir / f"{name}{suffix}.png"

            print(f"[{counter}/{total_images}] {output_path.name} (seed={seed})")
            t = time.time()
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
            result.images[0].save(output_path)
            print(f"           {time.time() - t:.1f}s -> {output_path}")

    print(f"\nTerminé. {counter} image(s) dans {args.output_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
