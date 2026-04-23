# Installation

Le générateur d'images Lelins tourne sur **macOS**, **Windows** et **Linux**.
Même code, même commandes — seule l'installation de PyTorch varie selon la plateforme.

## 1. Prérequis communs

- **Python 3.10 ou 3.11** (3.12 fonctionne mais certaines libs sont encore en rodage).
- **Git**.
- **15 Go d'espace disque libre** (les modèles SDXL pèsent ~7 Go, plus un cache).

Vérifier Python :

```bash
python3 --version
```

## 2. Cloner le dépôt

```bash
git clone https://github.com/julien-dao/lelins.git
cd lelins
```

## 3. Créer un environnement virtuel

### macOS / Linux

```bash
python3 -m venv .venv
source .venv/bin/activate
```

### Windows (PowerShell)

```powershell
python -m venv .venv
.venv\Scripts\Activate.ps1
```

## 4. Installer PyTorch (étape dépendante de la plateforme)

### macOS Apple Silicon (M1 / M2 / M3 / M4)

```bash
pip install --upgrade pip
pip install torch torchvision
```

L'accélération Metal (MPS) est incluse par défaut. Pas besoin d'index spécial.

### Windows / Linux avec GPU NVIDIA

CUDA 12.1 (le plus courant aujourd'hui) :

```bash
pip install --upgrade pip
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu121
```

Pour CUDA 11.8 ou une autre version, voir https://pytorch.org/get-started/locally/

### CPU uniquement (très lent, dépannage)

```bash
pip install --upgrade pip
pip install torch torchvision --index-url https://download.pytorch.org/whl/cpu
```

## 5. Installer les dépendances du projet

```bash
pip install -r scripts/requirements.txt
```

## 6. Vérifier que tout est bon

```bash
python scripts/generate.py --check
```

Ce qui doit afficher le périphérique détecté (`mps`, `cuda` ou `cpu`) sans rien
télécharger.

## 7. Premier lancement

La première génération télécharge **SDXL base** (~7 Go) depuis HuggingFace.
Prévoir 10-30 minutes selon la connexion. Les lancements suivants sont
instantanés car le modèle est en cache local.

```bash
python scripts/generate.py \
    --prompt "commercial product photography, male model wearing black boxer briefs, studio lighting, white seamless background" \
    --output outputs/test.png
```

## Dépannage

### macOS : `MPS backend out of memory`

Réduire la résolution :

```bash
python scripts/generate.py --prompt "..." --width 768 --height 1024
```

Ou activer le *attention slicing* (déjà actif par défaut sur Mac dans le script).

### Windows : `CUDA out of memory`

Idem, baisser la résolution, ou ajouter `--fp16` pour demi-précision.

### `No module named 'diffusers'`

L'environnement virtuel n'est pas activé. Relancer :

- macOS/Linux : `source .venv/bin/activate`
- Windows : `.venv\Scripts\Activate.ps1`

### Téléchargement du modèle interrompu

Supprimer le cache partiel et relancer :

- macOS/Linux : `rm -rf ~/.cache/huggingface`
- Windows : `rmdir /s %USERPROFILE%\.cache\huggingface`
