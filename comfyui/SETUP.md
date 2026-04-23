# ComfyUI — interface graphique pour le générateur Lelins

ComfyUI est une interface web nodale pour Stable Diffusion. Elle offre un contrôle
visuel (connexions de nœuds) plus pratique que la CLI pour l'exploration créative.
Ce guide explique comment l'installer sur **macOS** et **Windows**, puis charger
le workflow Lelins fourni.

## 1. Installer ComfyUI

### macOS (Apple Silicon)

```bash
# Dans un dossier parent (hors du dépôt Lelins)
git clone https://github.com/comfyanonymous/ComfyUI.git
cd ComfyUI
python3 -m venv venv
source venv/bin/activate
pip install --upgrade pip
pip install torch torchvision
pip install -r requirements.txt
```

### Windows (NVIDIA)

Le plus simple : **build portable officielle**.

1. Télécharger la release depuis https://github.com/comfyanonymous/ComfyUI/releases
   (fichier `ComfyUI_windows_portable_nvidia.7z`).
2. Extraire avec 7-Zip dans un dossier de ton choix.
3. Double-clic sur `run_nvidia_gpu.bat`.

## 2. Télécharger le modèle SDXL

ComfyUI attend les modèles dans `ComfyUI/models/checkpoints/`.

Télécharger **SDXL Base 1.0** depuis HuggingFace :

- URL : https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/blob/main/sd_xl_base_1.0.safetensors
- Taille : ~6.9 Go
- Placer dans : `ComfyUI/models/checkpoints/sd_xl_base_1.0.safetensors`

(Optionnel, améliore la qualité finale) **SDXL Refiner 1.0** :

- URL : https://huggingface.co/stabilityai/stable-diffusion-xl-refiner-1.0/blob/main/sd_xl_refiner_1.0.safetensors
- Placer dans : `ComfyUI/models/checkpoints/sd_xl_refiner_1.0.safetensors`

## 3. Lancer ComfyUI

### macOS / Linux

```bash
cd ComfyUI
source venv/bin/activate
python main.py
```

### Windows portable

Double-clic sur `run_nvidia_gpu.bat`.

Ouvrir http://127.0.0.1:8188 dans le navigateur.

## 4. Charger le workflow Lelins

1. Dans l'interface ComfyUI, cliquer sur **Load** (ou glisser-déposer le fichier).
2. Ouvrir `comfyui/workflows/lelins-sdxl-basic.json` depuis ce dépôt.
3. Le graphe de nœuds apparaît, pré-réglé pour SDXL + un prompt catalogue.
4. Modifier le texte dans le nœud **CLIP Text Encode (positive)** avec ton prompt.
5. Cliquer sur **Queue Prompt** en haut à droite.

Les images générées s'affichent dans le nœud **Save Image** et sont sauvegardées
dans `ComfyUI/output/`.

## 5. Astuces pour le catalogue Lelins

- **Seed** : dans le nœud KSampler, fixer la seed pour reproduire exactement un
  visuel qui te plaît, puis ne faire varier que le prompt (couleur, matière…).
- **Pose et corps cohérents** entre visuels : installer l'extension
  [ComfyUI-Manager](https://github.com/ltdrdata/ComfyUI-Manager) puis
  **IP-Adapter** ou **ControlNet** pour réutiliser un mannequin de référence.
- **Résolution** : SDXL est entraîné en 1024×1024. Pour un visuel portrait,
  utiliser 832×1216 ; pour paysage, 1216×832.

## 6. Lien avec le script Python

ComfyUI et le script `scripts/generate.py` utilisent exactement le **même
modèle**. Tu peux explorer un style dans ComfyUI (prompt, seed, réglages) puis
recopier les valeurs dans le script pour industrialiser la génération d'un
catalogue entier via `batch_generate.py`.
