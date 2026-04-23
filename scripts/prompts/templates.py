"""Templates de prompts pour le catalogue Lelins.

Structure un prompt SDXL à partir de paramètres produit simples (type, couleur,
matière, ambiance) pour maximiser la qualité et la cohérence visuelle sur toute
une série de visuels.
"""

from __future__ import annotations

from dataclasses import dataclass, field


DEFAULT_NEGATIVE = (
    "nude, naked, nsfw, explicit, genitals, "
    "deformed, disfigured, extra limbs, extra fingers, missing fingers, "
    "bad anatomy, bad proportions, mutated hands, "
    "low quality, worst quality, blurry, jpeg artifacts, watermark, "
    "logo, text, signature, cartoon, anime, 3d render, plastic skin"
)
"""Négatif partagé. On exclut explicitement la nudité : l'objectif est du visuel
catalogue propre, clothed product photography."""


QUALITY_SUFFIX = (
    "professional commercial product photography, studio lighting, soft key light, "
    "high detail fabric texture, sharp focus, photorealistic, 8k, "
    "shot on Canon EOS R5, 85mm lens, shallow depth of field"
)


BACKGROUNDS = {
    "studio_white": "plain white seamless studio background, clean, minimalist",
    "studio_grey": "soft grey seamless studio background, minimalist",
    "studio_beige": "warm beige seamless background, editorial",
    "loft": "modern loft interior, large window natural light, minimalist decor",
    "outdoor_beach": "sunny beach in golden hour, soft natural light, warm tones",
    "urban": "urban rooftop at sunset, modern architecture background, cinematic",
}


MODEL_STYLES = {
    "athletic": "athletic fit adult male model, toned physique, short hair, confident pose",
    "slim": "slim adult male model, natural look, friendly expression",
    "mature": "mature adult male model in his 40s, fit, grey at temples, elegant",
    "casual": "adult male model, casual everyday look, approachable",
}


@dataclass
class ProductPrompt:
    """Paramètres d'un visuel produit."""

    garment: str
    """Exemple: 'black cotton boxer briefs', 'navy blue trunks', 'white crew t-shirt'."""

    model_style: str = "athletic"
    """Clé de ``MODEL_STYLES``."""

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
        model_desc = MODEL_STYLES.get(self.model_style, self.model_style)
        bg_desc = BACKGROUNDS.get(self.background, self.background)

        parts = [
            model_desc,
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
