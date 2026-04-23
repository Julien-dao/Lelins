# Lelins

Vente en ligne de sous-vêtements et vêtements homme.

## Générateur d'images local

Ce dépôt contient un générateur d'images IA 100 % local, utilisable pour produire
des visuels de catalogue (mannequins portant les produits, ambiances, etc.).

- Moteur : **Stable Diffusion XL** (open-source, modèle exécuté localement).
- Cross-plateforme : **macOS** (Apple Silicon via MPS), **Windows** (NVIDIA via CUDA), **Linux**.
- Deux modes d'utilisation :
  - **ComfyUI** — interface graphique dans le navigateur pour l'exploration créative.
  - **Script Python** — CLI pour la génération automatisée (catalogue entier en un clic).

### Démarrage rapide

1. Lire [`docs/INSTALL.md`](docs/INSTALL.md) et installer les dépendances.
2. Lire [`docs/USAGE.md`](docs/USAGE.md) pour les commandes principales.
3. Pour l'interface graphique, lire [`comfyui/SETUP.md`](comfyui/SETUP.md).

### Structure

```
Lelins/
├── scripts/          Scripts Python (génération, batch, templates de prompts)
├── comfyui/          Setup ComfyUI + workflow prêt à l'emploi
├── docs/             Installation + utilisation
└── outputs/          Images générées (gitignored)
```
