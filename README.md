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

### Mannequins récurrents (Characters)

Chaque mannequin de la marque est défini dans un fichier JSON (`characters/`)
décrivant **40 paramètres** : âge, ethnicité, morphologie, peau, cheveux, yeux,
visage, corps, tatouages, piercings, marques distinctives. Une fois généré,
son **identité est verrouillée par seed** et il peut être réutilisé sur tous
les visuels du catalogue.

Plusieurs mannequins peuvent coexister (ligne principale, ligne mature, etc.).

### Démarrage rapide

1. Lire [`docs/INSTALL.md`](docs/INSTALL.md) et installer les dépendances.
2. Lire [`docs/USAGE.md`](docs/USAGE.md) pour les commandes principales.
3. Pour l'interface graphique, lire [`comfyui/SETUP.md`](comfyui/SETUP.md).
4. Pour les profils mannequin, lire [`characters/README.md`](characters/README.md).

### Structure

```
Lelins/
├── scripts/          Scripts Python (generate, batch, generate_character)
│   └── prompts/      Character (40 params) + templates produit
├── characters/       Profils mannequin JSON + portraits de référence
├── comfyui/          Setup ComfyUI + workflow prêt à l'emploi
├── docs/             Installation + utilisation
└── outputs/          Images générées (gitignored)
```
