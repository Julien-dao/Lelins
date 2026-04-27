"""Helpers IP-Adapter pour SDXL et SD 1.5.

IP-Adapter permet d'**injecter une image de référence** dans un pipeline :
- Adapter "face" : verrouille l'identité d'un visage à travers plusieurs visuels
  (cohérence du mannequin Lelins).
- Adapter "general" : transfère le style/contenu d'une image de référence
  (utilisé par tryon.py pour transférer l'apparence d'un vêtement réel).

Les poids sont téléchargés depuis ``h94/IP-Adapter`` au premier appel et mis
en cache localement.
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image


IP_ADAPTER_REPO = "h94/IP-Adapter"

# SDXL — sous-dossier sdxl_models, encoder ViT-H/14
SDXL_SUBFOLDER = "sdxl_models"
SDXL_FACE_WEIGHT = "ip-adapter-plus-face_sdxl_vit-h.safetensors"
SDXL_GENERAL_WEIGHT = "ip-adapter_sdxl.safetensors"

# SD 1.5 — sous-dossier models, encoder ViT-H/14
SD15_SUBFOLDER = "models"
SD15_FACE_WEIGHT = "ip-adapter-plus-face_sd15.bin"
SD15_GENERAL_WEIGHT = "ip-adapter_sd15.bin"


def _is_sdxl_pipe(pipe) -> bool:
    """Détecte si un pipeline est SDXL (vs SD 1.5) d'après sa classe."""
    return type(pipe).__name__.startswith("StableDiffusionXL")


def load_face_adapter(pipe, scale: float = 0.8) -> None:
    """Charge l'adapter visage et règle son intensité.

    Choisit automatiquement les bons poids selon que le pipeline est SDXL
    ou SD 1.5.

    ``scale`` contrôle à quel point l'identité de référence prime sur le prompt :
    - 0.6 : équilibré, garde une part d'aléatoire (recommandé pour la 1ère essai)
    - 0.8 : verrouillage net (par défaut)
    - 1.0 : verrouillage strict, peut figer aussi pose et angle
    """
    if _is_sdxl_pipe(pipe):
        subfolder, weight = SDXL_SUBFOLDER, SDXL_FACE_WEIGHT
    else:
        subfolder, weight = SD15_SUBFOLDER, SD15_FACE_WEIGHT
    pipe.load_ip_adapter(IP_ADAPTER_REPO, subfolder=subfolder, weight_name=weight)
    pipe.set_ip_adapter_scale(scale)


def load_general_adapter(pipe, scale: float = 0.7) -> None:
    """Charge l'adapter général (utilisé pour le transfert vêtement)."""
    if _is_sdxl_pipe(pipe):
        subfolder, weight = SDXL_SUBFOLDER, SDXL_GENERAL_WEIGHT
    else:
        subfolder, weight = SD15_SUBFOLDER, SD15_GENERAL_WEIGHT
    pipe.load_ip_adapter(IP_ADAPTER_REPO, subfolder=subfolder, weight_name=weight)
    pipe.set_ip_adapter_scale(scale)


def open_reference_image(path: Path | str, max_side: int = 1024) -> Image.Image:
    """Charge une image de référence et la borne à ``max_side`` pixels.

    IP-Adapter est plus stable avec des références ~512-1024 px. On évite
    surtout les images démesurées qui consomment inutilement de la mémoire.
    """
    img = Image.open(path).convert("RGB")
    w, h = img.size
    if max(w, h) > max_side:
        scale = max_side / max(w, h)
        img = img.resize((int(w * scale), int(h * scale)), Image.LANCZOS)
    return img
