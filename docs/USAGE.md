# Utilisation du générateur Lelins

Trois CLI principaux, **totalement interopérables** (même modèle SDXL) :

| Script                       | Rôle                                                        |
|------------------------------|-------------------------------------------------------------|
| `generate_character.py`      | Crée le **portrait** d'un mannequin (Character JSON).       |
| `generate.py`                | Génère **un visuel produit** (mannequin + vêtement + décor). |
| `batch_generate.py`          | Génère **tout un catalogue** depuis un JSON descriptif.     |

L'interface graphique ComfyUI est documentée à part : [`comfyui/SETUP.md`](../comfyui/SETUP.md).
L'installation est couverte dans [`INSTALL.md`](INSTALL.md).

---

## Workflow recommandé : créer puis utiliser un mannequin

### Étape 1. Définir le mannequin

Copier le profil exemple et l'éditer :

```bash
cp characters/lelins-main.example.json characters/lelins-main.json
# puis ouvrir characters/lelins-main.json dans ton éditeur
```

Ce JSON expose **40 paramètres** : âge, ethnicité, morphologie, peau, cheveux,
yeux, nez, bouche, mâchoire, corps, tatouages, piercings, marques distinctives.
Voir `scripts/prompts/character.py` pour la liste exhaustive des valeurs
acceptées.

### Étape 2. Générer son portrait

```bash
python scripts/generate_character.py \
    --profile characters/lelins-main.json \
    --output characters/lelins-main.png
```

Au premier run, une **seed aléatoire** est tirée puis enregistrée dans le JSON
(`"seed": 12345`). C'est ce qui assurera la cohérence du visage par la suite.

**Itérations sur le portrait :**

- Ajuster un trait dans le JSON (ex. `eye_color: "blue"` → `"green"`) et
  relancer la même commande → la seed est conservée, seul le trait modifié
  bouge nettement.
- Pas content du visage de base ? Tirer une nouvelle seed :

```bash
python scripts/generate_character.py \
    --profile characters/lelins-main.json --reroll \
    --output characters/lelins-main.png
```

### Étape 3. Habiller le mannequin

Une fois le visage validé, on l'utilise pour générer des visuels produits :

```bash
python scripts/generate.py \
    --character characters/lelins-main.json \
    --garment "black cotton boxer briefs" \
    --background studio_white \
    --pose "standing front view, arms relaxed at sides" \
    --output outputs/main-boxer-noir.png
```

La seed du Character est **automatiquement réutilisée** → le mannequin reste
identique d'un visuel à l'autre.

### Étape 4. Catalogue complet

Lister tous les visuels voulus dans un JSON, puis lancer un seul batch.
Exemple fourni : `scripts/prompts/examples.json`.

```bash
python scripts/batch_generate.py \
    --input scripts/prompts/examples.json \
    --output-dir outputs/catalog
```

Le modèle ne se charge qu'une fois pour tout le lot — bien plus rapide qu'une
boucle de `generate.py`.

---

## Plusieurs mannequins ?

Aucun problème : crée autant de fichiers `characters/<nom>.json` que tu veux.
Chaque profil a sa propre seed, donc sa propre identité.

```bash
cp characters/lelins-mature.example.json characters/lelins-mature.json
python scripts/generate_character.py \
    --profile characters/lelins-mature.json \
    --output characters/lelins-mature.png
```

Dans le batch, le champ `"character"` de chaque item pointe vers le profil voulu :

```json
{
  "name": "ligne-mature-boxer-gris",
  "character": "characters/lelins-mature.json",
  "garment": "heather grey boxer briefs",
  "background": "loft"
}
```

---

## Référence des CLI

### `generate.py` — un visuel produit

```bash
python scripts/generate.py [options]
```

| Option            | Description                                                       |
|-------------------|-------------------------------------------------------------------|
| `--character` / `-c` | Profil JSON du mannequin (40 paramètres). Prioritaire.         |
| `--garment`       | Vêtement porté (texte libre).                                     |
| `--model-style`   | Preset simple si pas de Character (`athletic`/`slim`/`mature`/`casual`). |
| `--background`    | Décor (`studio_white`/`studio_grey`/`studio_beige`/`loft`/`outdoor_beach`/`urban`). |
| `--pose`          | Pose (texte libre).                                               |
| `--prompt`        | Prompt libre complet (ignore tout le reste).                      |
| `--negative`      | Prompt négatif (défaut : exclusions de qualité standard).         |
| `--width` / `--height` | Résolution (défaut 1024×1024).                              |
| `--steps`         | Pas de diffusion (défaut 30, plage 20-40).                        |
| `--cfg`           | Force du prompt (défaut 7, plage 5-9).                            |
| `--seed`          | Seed reproductible (sinon : seed du Character ou aléatoire).      |
| `--output` / `-o` | Chemin de sortie de l'image.                                      |
| `--check`         | Affiche le matériel détecté et quitte.                            |

### `generate_character.py` — portrait mannequin

```bash
python scripts/generate_character.py --profile <json> --output <png> [options]
```

| Option       | Description                                                  |
|--------------|--------------------------------------------------------------|
| `--profile`  | Profil JSON du mannequin (obligatoire).                      |
| `--output`   | Chemin de sortie du portrait (obligatoire).                  |
| `--reroll`   | Tire une nouvelle seed (écrase celle du JSON après succès).  |
| `--seed`     | Force une seed précise (prioritaire sur `--reroll`).         |
| `--width` / `--height` | Résolution (défaut 896×1152, format portrait).     |

### `batch_generate.py` — catalogue complet

```bash
python scripts/batch_generate.py --input <json> --output-dir <dir> [options]
```

Format d'un item :

```json
{
  "name": "fichier-de-sortie",
  "character": "characters/lelins-main.json",
  "garment": "navy blue trunks",
  "background": "outdoor_beach",
  "pose": "standing on the beach",
  "seed": null,
  "variations": 2
}
```

- `character` (recommandé) : chemin vers un profil. Sa seed enregistrée est
  réutilisée si l'item ne précise pas la sienne.
- `seed` au niveau de l'item : prioritaire sur la seed du Character.
- `variations` : nombre d'images (chacune avec seed + 0, +1, +2…).
- Alternative : remplacer `garment` par `prompt` pour un contrôle total.

---

## Durées typiques

Indicatif, premier run hors téléchargement, en 1024×1024 / 30 steps :

| Matériel                         | Durée par image            |
|----------------------------------|----------------------------|
| RTX 4090 (CUDA)                  | ~4 s                       |
| RTX 3070 8 Go (CUDA)             | ~12 s                      |
| Apple M3 Max (MPS)               | ~20 s                      |
| Apple M2 16 Go (MPS)             | ~45 s                      |
| Apple M1 8 Go (MPS)              | ~90 s (baisser à 768×768)  |
| CPU (dépannage)                  | 5-10 min                   |

---

## Conseils qualité

- Préférer des prompts en **anglais** : SDXL est entraîné principalement dessus,
  les résultats sont nettement meilleurs.
- **Matière et détails** dans le prompt : "cotton", "microfiber",
  "ribbed fabric", "elastic waistband" améliorent le rendu textile.
- **Éclairage** : "soft key light", "golden hour", "north window light" font une
  vraie différence.
- **Mains déformées** (classique SDXL) : ajouter "hands out of frame" ou
  "hands in pockets" à la pose, ou re-générer avec une seed légèrement
  différente.
- **Cohérence du mannequin** entre visuels : passer par un Character JSON
  avec sa seed enregistrée. Pour une identité **strictement** verrouillée,
  prochaine itération : intégration de IP-Adapter (visage figé sur image de
  référence).
