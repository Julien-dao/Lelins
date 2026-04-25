# Essayage virtuel — habiller un mannequin avec un produit réel

Ce document explique comment uploader la **photo réelle** d'un sous-vêtement
(ou autre vêtement) et la voir portée par un mannequin Lelins.

## Approche utilisée

Le pipeline `tryon.py` combine deux briques **diffusers natives** :

1. **SDXL Inpainting** — re-génère la zone du corps couverte par un masque.
2. **IP-Adapter (général)** — guide cette re-génération avec l'image du
   vêtement réel, pour reproduire au mieux sa couleur, sa coupe et sa texture.

Quatre éléments à fournir :

| Entrée            | Description                                                     |
|-------------------|-----------------------------------------------------------------|
| `--person`        | Image du mannequin (issue de `generate.py`).                    |
| `--garment`       | Photo packshot du vêtement (idéalement fond blanc, plein cadre). |
| `--mask`          | Masque PNG : blanc = re-générer, noir = conserver.              |
| `--prompt`        | Description courte du vêtement (couleur, matière, coupe).       |

## Workflow complet

### 1. Générer un mannequin habillé "neutre"

On a besoin d'une image de personne **portant déjà quelque chose** dans la zone
qu'on veut remplacer. Lance par exemple :

```bash
python scripts/generate.py \
    --character characters/lelins-main.json \
    --garment "plain neutral grey boxer briefs" \
    --background studio_white \
    --output outputs/main-base.png
```

C'est l'image personne de base, qui va servir d'ancrage pour le try-on.

### 2. Créer le masque

Le masque définit où ré-habiller. Pour des sous-vêtements, l'heuristique
auto-hip suffit dans la plupart des cas :

```bash
python scripts/make_mask.py \
    --person outputs/main-base.png \
    --auto-hip \
    --output masks/boxer-zone.png
```

Pour un t-shirt → `--auto-torso`.
Pour un masque précis → l'ouvrir dans Photoshop / GIMP / Photopea, peindre
en blanc la zone du vêtement (avec un peu de marge), exporter en PNG.

### 3. Lancer l'essayage virtuel

```bash
python scripts/tryon.py \
    --person outputs/main-base.png \
    --garment photos/produits/boxer-noir-coton.jpg \
    --mask masks/boxer-zone.png \
    --prompt "black cotton boxer briefs with elastic waistband and Lelins logo" \
    --output outputs/main-portant-boxer-noir.png
```

Le mannequin garde son visage, sa peau, son corps — seule la zone masquée est
re-générée pour ressembler à ton produit.

## Réglages

### `--garment-scale` (0.5 - 1.0, défaut 0.85)

Contrôle à quel point le vêtement de référence guide le rendu :

- **0.5-0.6** : prompt textuel prime ; le vêtement de réf donne juste une
  ambiance générale. Utile si la photo packshot est de mauvaise qualité.
- **0.85** (défaut) : équilibre — la coupe et la couleur du produit réel
  ressortent fidèlement.
- **0.95-1.0** : verrouillage maximum. Peut produire des artéfacts si la
  photo ne ressemble pas à ce qu'un humain pourrait porter (vêtement à plat
  étiré, par ex.).

### `--strength` (0.7 - 1.0, défaut 0.95)

Contrôle l'intensité du repaint dans la zone masquée :

- **1.0** : redessine entièrement (recommandé pour bien remplacer un autre
  vêtement existant).
- **0.7-0.85** : conserve une partie de la zone d'origine. Utile pour des
  ajustements fins.

### `--steps` et `--cfg`

Plus de pas (40-50) = plus de détails textile, plus lent.
`--cfg 8` rend le rendu plus fidèle au prompt textuel mais peut écraser le
guidage IP-Adapter.

## Conseils pratiques

### Pour la photo packshot du vêtement

- **Fond uni** (blanc, gris clair) — supprime les distractions.
- **Vêtement à plat ou sur cintre** > vêtement froissé.
- **Bonne résolution** (>= 1024 px du grand côté). Au-delà de 2048 px, le
  helper redimensionne automatiquement.
- **Frontal** — les vues 3/4 fonctionnent moins bien pour le transfert.

### Pour le masque

- **Trop large** : le système redessine du corps qui n'aurait pas dû changer
  (peau, cuisses, abdomen) → mannequin différent.
- **Trop étroit** : on voit la jonction entre la zone régénérée et l'ancien
  vêtement.
- **Bord adouci** (`--feather 10` à `20` dans `make_mask.py`) — meilleurs raccords.
- **Couvrir les marges** du vêtement original avec un peu de débord pour
  effacer les anciennes coutures.

### Pour le prompt

Rester **court et précis** sur le vêtement. Mots-clés efficaces :

- Couleurs : "deep navy blue", "heather grey", "off-white"
- Matières : "ribbed cotton", "smooth microfiber", "stretch jersey"
- Détails : "elastic waistband", "flat lock seams", "branded waistband",
  "trunk cut", "low rise"

Éviter de re-décrire la personne (le mannequin) — c'est l'image source qui le
définit déjà.

## Limites de cette approche

- **Logos / textes** sur le vêtement : pas reproduits avec fidélité (SDXL ne
  copie pas pixel à pixel). Pour un logo précis et lisible, prévoir une étape
  de retouche manuelle (Photoshop) ou utiliser **CatVTON** (voir ci-dessous).
- **Motifs complexes** (rayures fines, prints) : approximés. Marche mieux pour
  des unis et des aplats simples.
- **Coupe inhabituelle** : si la photo packshot montre un vêtement très
  différent du masque (slip vs boxer long), le système peut produire des
  proportions étranges.

## Option avancée : CatVTON / IDM-VTON

Pour une **fidélité supérieure** sur les logos, prints et motifs, les modèles
spécialisés en virtual try-on existent :

- **CatVTON** — https://github.com/Zheng-Chong/CatVTON (le plus léger, ~6 Go)
- **IDM-VTON** — https://github.com/yisol/IDM-VTON (qualité top, ~16 Go VRAM)
- **OOTDiffusion** — https://github.com/levihsu/OOTDiffusion

Ils ne sont pas intégrés ici car leur installation sort du cadre `pip install`
standard (cloning manuel, modèles à télécharger séparément, dépendances de
pose-estimation). Si la qualité actuelle ne suffit pas pour ton catalogue, on
peut les ajouter dans une itération suivante avec un script wrapper dédié.

## Durées typiques (Mac MPS)

Indicatif, image 1024×1024 :

| Étape                       | M3 Max | M2 16 Go | M1 8 Go            |
|-----------------------------|--------|----------|--------------------|
| `tryon.py` (35 steps)       | ~30 s  | ~70 s    | ~150 s (768×768 conseillé) |
| `make_mask.py`              | <1 s   | <1 s     | <1 s               |

## Récapitulatif minimal

```bash
# 1. Mannequin de base (une fois suffit)
python scripts/generate.py -c characters/lelins-main.json \
    --garment "plain grey boxer briefs" --background studio_white \
    -o outputs/main-base.png

# 2. Masque de la zone à habiller
python scripts/make_mask.py --person outputs/main-base.png \
    --auto-hip -o masks/boxer-zone.png

# 3. Try-on avec ton produit réel
python scripts/tryon.py \
    --person outputs/main-base.png \
    --garment photos/boxer-noir.jpg \
    --mask masks/boxer-zone.png \
    --prompt "black cotton boxer briefs, branded elastic waistband" \
    -o outputs/main-portant-boxer-noir.png
```
