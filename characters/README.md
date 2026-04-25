# Characters — profils mannequin Lelins

Ce dossier contient les profils JSON des mannequins récurrents de la marque.
Chaque profil décrit un humain via **40 paramètres** (peau, cheveux, yeux,
visage, corps, tatouages, piercings, marques distinctives) et est réutilisable
dans toute l'application.

## Profils fournis

| Fichier                          | Description                                    |
|----------------------------------|------------------------------------------------|
| `lelins-main.example.json`       | Mannequin principal — méditerranéen, 28 ans, athlétique. |
| `lelins-mature.example.json`     | Mannequin pour la ligne mature — européen, 45 ans, salt-and-pepper, tatoué. |

Ce sont des **exemples**. Pour démarrer, copie l'un d'eux sans le suffixe
`.example` :

```bash
cp characters/lelins-main.example.json characters/lelins-main.json
```

Puis édite-le à ton goût avant de générer.

## Workflow type

### 1. Générer le portrait pour valider l'identité

```bash
python scripts/generate_character.py \
    --profile characters/lelins-main.json \
    --output characters/lelins-main.png
```

Au premier lancement, une **seed aléatoire** est tirée et **enregistrée** dans
le JSON (`"seed": 12345`). Toutes les générations futures qui réutilisent ce
profil partiront de cette seed → le visage reste cohérent.

### 2. Itérer sur le portrait

Pas content du visage ? Deux possibilités :

- **Changer un paramètre** dans le JSON et relancer (la seed est conservée,
  seul le trait modifié change visiblement).
- **Reroll de seed** pour partir sur un autre visage tout en gardant la même
  description :

```bash
python scripts/generate_character.py \
    --profile characters/lelins-main.json --reroll \
    --output characters/lelins-main.png
```

### 3. Utiliser le mannequin pour générer un produit

```bash
python scripts/generate.py \
    --character characters/lelins-main.json \
    --garment "black cotton boxer briefs" \
    --background studio_white \
    --output outputs/main-boxer-noir.png
```

La seed du Character est automatiquement réutilisée → le mannequin reste
identique d'un produit à l'autre.

### 4. Catalogue complet

Voir `scripts/prompts/examples.json` pour un fichier d'entrée batch qui
référence un Character. Lancer :

```bash
python scripts/batch_generate.py \
    --input scripts/prompts/examples.json \
    --output-dir outputs/catalog
```

## Référence des champs

Voir `scripts/prompts/character.py` pour la liste complète et les valeurs
acceptées de chaque paramètre (exemple : `hair_style` accepte `classic_short`,
`side_part`, `slicked_back`, `messy`, `textured_crop`, `curly`, `wavy`, `afro`,
`dreadlocks`, `braids`, `man_bun`, `undercut`, `fade`, `mohawk`).

Les vocabulaires fermés sont validés en lecture mais le code reste tolérant si
tu fournis une valeur libre — elle sera passée telle quelle au prompt SDXL.

## Verrouillage du visage (IP-Adapter)

Une fois le portrait généré (étape 2), le `portrait_path` est enregistré dans
le profil JSON. Toutes les générations suivantes (`generate.py`,
`batch_generate.py`) qui réfèrent ce Character **utilisent automatiquement
l'IP-Adapter face** : le visage du portrait sert de référence forcée et reste
quasi-identique d'un visuel à l'autre.

Pour ajuster la force du verrouillage :

```bash
# Verrouillage strict (recommandé pour catalogue uniformisé)
python scripts/generate.py -c characters/lelins-main.json \
    --garment "..." --face-scale 0.9 -o outputs/x.png

# Désactiver complètement
python scripts/generate.py -c characters/lelins-main.json \
    --garment "..." --no-face-lock -o outputs/x.png
```

Au premier usage, l'IP-Adapter télécharge ~2 Go (poids + image encoder ViT-H/14)
puis cache pour les runs suivants.
