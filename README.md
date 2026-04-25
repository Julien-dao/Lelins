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

### Trois phases du pipeline

1. **Mannequin paramétré** — Character JSON à 40 attributs (peau, cheveux,
   yeux, visage, corps, tatouages…) → portrait studio reproductible.
2. **Verrouillage facial (IP-Adapter)** — le visage du mannequin reste
   identique sur toutes les images du catalogue.
3. **Essayage virtuel** — uploader la photo réelle d'un produit (boxer, slip,
   t-shirt…), le système l'applique sur le mannequin via SDXL Inpainting +
   IP-Adapter. Voir [`docs/TRYON.md`](docs/TRYON.md).

### Démarrage rapide

1. Lire [`docs/INSTALL.md`](docs/INSTALL.md) et installer les dépendances.
2. Lire [`docs/USAGE.md`](docs/USAGE.md) pour les commandes principales.
3. Pour l'essayage virtuel, lire [`docs/TRYON.md`](docs/TRYON.md).
4. Pour l'interface graphique, lire [`comfyui/SETUP.md`](comfyui/SETUP.md).
5. Pour les profils mannequin, lire [`characters/README.md`](characters/README.md).

### Structure

```
Lelins/
├── scripts/
│   ├── generate.py            Visuel produit (mannequin + vêtement + décor)
│   ├── generate_character.py  Portrait d'un Character
│   ├── batch_generate.py      Catalogue complet en un lot
│   ├── tryon.py               Essayage virtuel (vêtement réel sur mannequin)
│   ├── make_mask.py           Création de masques pour tryon.py
│   ├── ip_adapter_helpers.py  Chargement IP-Adapter face/general
│   └── prompts/
│       ├── character.py       Dataclass Character (40 paramètres)
│       └── templates.py       Composition Character + garment + scène
├── characters/                Profils mannequin + portraits de référence
├── comfyui/                   Setup ComfyUI + workflow prêt à l'emploi
├── docs/                      INSTALL.md, USAGE.md, TRYON.md
└── outputs/                   Images générées (gitignored)
```
