"""Composition de prompts SDXL pour le catalogue Lelins.

Trois éléments combinables :

- un ``Character`` (description fine du mannequin, voir ``character.py``) ou,
  à défaut, un ``MODEL_STYLES`` simplifié (legacy, pour rétrocompatibilité) ;
- un ``garment`` (texte libre décrivant le produit porté) ;
- un ``background`` (clé de ``BACKGROUNDS``).

Le résultat est ``(prompt_positif, prompt_négatif)``.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Optional

from .character import Character


DEFAULT_NEGATIVE = (
    "nude, naked, nsfw, explicit, genitals, "
    "deformed, disfigured, extra limbs, extra fingers, missing fingers, "
    "bad anatomy, bad proportions, mutated hands, "
    "low quality, worst quality, blurry, jpeg artifacts, watermark, "
    "logo, text, signature, cartoon, anime, 3d render, plastic skin"
)
"""Négatif partagé. Visuel catalogue propre, photographie habillée."""


QUALITY_SUFFIX = (
    "fashion editorial photography, magazine cover quality, "
    "cinematic lighting, professional color grading, "
    "natural skin tones, detailed skin texture, perfect anatomy, "
    "high detail fabric, sharp focus on subject, blurred background bokeh, "
    "shot on Canon EOS R5, 85mm f/1.4 lens, shallow depth of field, "
    "photorealistic, 8k, ultra detailed, masterpiece"
)


BACKGROUNDS = {
    "studio_white": "plain white seamless studio background, clean, minimalist, "
        "even softbox lighting, high-key fashion photography",
    "studio_grey": "soft grey seamless studio background, minimalist, "
        "controlled studio lighting, fashion catalog look",
    "studio_beige": "warm beige seamless background, editorial fashion, "
        "soft directional light",
    "loft": "modern loft interior, large window natural light streaming in, "
        "minimalist decor, morning sunshine, lifestyle photography",
    "outdoor_beach": "tropical beach at golden hour, turquoise water, palm trees in background, "
        "wet sand reflecting sunset, warm golden light, "
        "cinematic ocean horizon, summer vibes",
    "outdoor_beach_walking": "walking on tropical beach at golden hour, mid-stride pose, "
        "palm trees silhouette, turquoise ocean, soft waves on sand, "
        "warm sunset backlight, hair gently moving in sea breeze, "
        "cinematic motion, fashion editorial, slow-motion feel",
    "urban": "urban rooftop at sunset, modern architecture background, "
        "city skyline, cinematic warm light, fashion editorial",
    "pool": "luxury pool deck at golden hour, turquoise water reflections, "
        "modern minimalist architecture, warm summer light",
}


# Profils mannequin simplifiés — utilisés uniquement si aucun ``Character``
# n'est fourni. Pour un mannequin récurrent et richement paramétré, utiliser
# ``Character`` (40 paramètres) plutôt que ces presets.
MODEL_STYLES = {
    "athletic": "athletic fit adult male model, toned physique, short hair, confident pose",
    "slim": "slim adult male model, natural look, friendly expression",
    "mature": "mature adult male model in his 40s, fit, grey at temples, elegant",
    "casual": "adult male model, casual everyday look, approachable",
}


@dataclass
class ProductPrompt:
    """Paramètres d'un visuel produit.

    Si ``character`` est fourni, c'est lui qui décrit le mannequin (40 params).
    Sinon on retombe sur ``model_style`` (preset simple).
    """

    garment: str
    """Exemple: 'black cotton boxer briefs', 'navy blue trunks', 'white crew t-shirt'."""

    character: Optional[Character] = None
    """Description fine du mannequin. Prioritaire sur ``model_style`` si défini."""

    model_style: str = "athletic"
    """Preset legacy, utilisé si ``character`` n'est pas fourni."""

    background: str = "studio_white"
    """Clé de ``BACKGROUNDS``."""

    pose: str = "standing front view, arms relaxed at sides"
    """Description de la pose."""

    extra: list[str] = field(default_factory=list)
    """Détails additionnels libres ajoutés au prompt positif."""

    negative_extra: list[str] = field(default_factory=list)
    """Exclusions additionnelles propres à ce visuel."""

    def build(self) -> tuple[str, str]:
        """Retourne (positive_prompt, negative_prompt)."""
        if self.character is not None:
            person_desc = self.character.build_prompt()
        else:
            person_desc = MODEL_STYLES.get(self.model_style, self.model_style)

        bg_desc = BACKGROUNDS.get(self.background, self.background)

        parts = [
            person_desc,
            f"wearing {self.garment}",
            self.pose,
            bg_desc,
            *self.extra,
            QUALITY_SUFFIX,
        ]
        positive = ", ".join(p for p in parts if p)

        negative_parts = [DEFAULT_NEGATIVE, *self.negative_extra]
        negative = ", ".join(n for n in negative_parts if n)

        return positive, negative


def simple_prompt(garment: str, **kwargs) -> tuple[str, str]:
    """Raccourci : construit un ProductPrompt et retourne (positif, négatif)."""
    return ProductPrompt(garment=garment, **kwargs).build()
