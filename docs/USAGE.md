# Utilisation du générateur Lelins

Deux modes au choix, **totalement interopérables** (même modèle SDXL) :

1. **Script Python** — `scripts/generate.py` et `scripts/batch_generate.py`,
   parfait pour générer rapidement un visuel ou automatiser tout un catalogue.
2. **ComfyUI** — interface graphique pour explorer et peaufiner. Voir
   [`comfyui/SETUP.md`](../comfyui/SETUP.md).

L'installation est couverte dans [`docs/INSTALL.md`](INSTALL.md).

---

## 1. Vérifier l'installation

```bash
python scripts/generate.py --check
```

Affiche le matériel détecté (`mps`, `cuda` ou `cpu`). Ne télécharge rien.

## 2. Générer un visuel unique (mode template)

Mode le plus simple — tu décris le produit, le style de mannequin et le décor,
le script construit un prompt optimisé pour toi :

```bash
python scripts/generate.py \
    --garment "black cotton boxer briefs" \
    --model-style athletic \
    --background studio_white \
    --output outputs/boxer-noir.png
```

**Options disponibles**

- `--garment` : description libre du vêtement (obligatoire).
- `--model-style` : `athletic` | `slim` | `mature` | `casual`.
- `--background` : `studio_white` | `studio_grey` | `studio_beige` | `loft`
  | `outdoor_beach` | `urban`.
- `--pose` : description libre de la pose.

## 3. Générer avec un prompt libre

Pour un contrôle total du prompt :

```bash
python scripts/generate.py \
    --prompt "editorial photo of a mature male model wearing a heather grey crew neck t-shirt, loft interior, morning light" \
    --output outputs/editorial.png
```

Tu peux aussi fournir `--negative "..."` pour remplacer le négatif par défaut.

## 4. Paramètres avancés

- `--width 832 --height 1216` — format portrait (par défaut : 1024×1024 carré).
- `--steps 40` — plus de pas = plus de détails, plus lent (par défaut : 30).
- `--cfg 8` — force d'adhésion au prompt (par défaut : 7, plage utile 5-9).
- `--seed 42` — rend la génération **reproductible** (sans seed, chaque lancement
  varie). Indispensable pour itérer sur un visuel qui te plaît.

## 5. Générer un catalogue entier (batch)

Décrire l'ensemble des visuels voulus dans un JSON, puis lancer en une commande.
Un exemple complet est fourni : `scripts/prompts/examples.json`.

```bash
python scripts/batch_generate.py \
    --input scripts/prompts/examples.json \
    --output-dir outputs/catalog
```

**Format d'un item**

```json
{
  "name": "boxer-noir-coton",
  "garment": "black cotton boxer briefs",
  "model_style": "athletic",
  "background": "studio_white",
  "pose": "standing front view, arms relaxed at sides",
  "seed": 1001,
  "variations": 3
}
```

- `variations` : combien d'images générer pour cet item (chacune avec une seed
  légèrement différente).
- Alternative : remplacer `garment` par `prompt` pour un contrôle total.

Le modèle n'est chargé **qu'une seule fois** pour tout le lot — bien plus rapide
qu'appeler `generate.py` en boucle.

## 6. Flux de travail recommandé

1. **Exploration** dans ComfyUI : tester des styles, mannequins, poses
   jusqu'à trouver un rendu qui te plaît.
2. **Capture** de la seed + des réglages (steps, cfg, prompt) dans ComfyUI.
3. **Industrialisation** : transposer dans un fichier JSON, lancer
   `batch_generate.py` pour produire tout ton catalogue avec une seed stable
   → cohérence visuelle entre les produits.

## 7. Durées typiques

Indicatif, premier lancement hors téléchargement :

| Matériel                         | 1024×1024, 30 steps |
|----------------------------------|---------------------|
| RTX 4090 (CUDA)                  | ~4 s                |
| RTX 3070 8 Go (CUDA)             | ~12 s               |
| Apple M3 Max (MPS)               | ~20 s               |
| Apple M2 16 Go (MPS)             | ~45 s               |
| Apple M1 8 Go (MPS)              | ~90 s (baisser à 768×768) |
| CPU (dépannage)                  | 5-10 min            |

## 8. Conseils qualité

- Préférer des prompts en **anglais** : SDXL est entraîné principalement dessus,
  les résultats sont nettement meilleurs.
- **Matière et détails** dans le prompt : mentionner "cotton", "microfiber",
  "ribbed fabric", "elastic waistband" améliore le rendu textile.
- **Éclairage** : "soft key light", "golden hour", "north window light" font une
  vraie différence.
- Si les **mains sont déformées** (classique SDXL), essayer d'ajouter "hands
  out of frame" ou "hands in pockets" à la pose, ou générer une variation
  (changer `--seed`).
- Pour une **cohérence de mannequin** entre visuels : fixer `--seed`, ne changer
  que le produit dans le prompt. Pour une identité stricte, passer par
  ComfyUI + IP-Adapter (voir `comfyui/SETUP.md`).
