#!/usr/bin/env python3
"""Lance le serveur web Lelins en local.

Usage typique :

    # Local uniquement (depuis le Mac, ouvert sur http://127.0.0.1:8000)
    python scripts/serve.py

    # Accessible depuis le réseau local (téléphone sur même Wi-Fi)
    python scripts/serve.py --host 0.0.0.0

    # Port personnalisé
    python scripts/serve.py --port 8080

Le pipeline SDXL est chargé paresseusement à la première requête. Le serveur
démarre en ~1 s ; la première génération prend 30-90 s (chargement modèle),
les suivantes ~10-60 s selon le matériel.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Serveur web Lelins (FastAPI + Uvicorn)")
    p.add_argument("--host", default="127.0.0.1",
                   help="Adresse d'écoute (127.0.0.1 = local seulement, "
                        "0.0.0.0 = accessible depuis le LAN). Défaut : 127.0.0.1")
    p.add_argument("--port", type=int, default=8000,
                   help="Port d'écoute. Défaut : 8000")
    p.add_argument("--reload", action="store_true",
                   help="Auto-reload du code (dev seulement)")
    return p.parse_args()


def main() -> int:
    args = parse_args()

    try:
        import uvicorn  # noqa: F401
    except ImportError:
        print("ERREUR : uvicorn n'est pas installé.", file=sys.stderr)
        print("Installer avec : pip install -r scripts/requirements.txt", file=sys.stderr)
        return 1

    # On ajoute la racine du projet au path pour que `from server.app import app` marche.
    sys.path.insert(0, str(ROOT))

    print(f"Serveur Lelins")
    print(f"  → http://{args.host}:{args.port}")
    if args.host == "127.0.0.1":
        print(f"  (local uniquement ; pour accéder depuis le téléphone, relancer avec --host 0.0.0.0)")
    print()

    import uvicorn
    uvicorn.run(
        "server.app:app",
        host=args.host,
        port=args.port,
        reload=args.reload,
        app_dir=str(ROOT),
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
