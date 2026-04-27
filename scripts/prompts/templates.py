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
    # === Studio (clean, focus produit) ===
    "studio_white": "plain white seamless studio background, clean minimalist, "
        "even softbox lighting, high-key fashion photography",
    "studio_grey": "soft grey seamless studio background, controlled studio lighting, "
        "professional fashion catalog look",
    "studio_beige": "warm beige seamless background, soft directional light, "
        "editorial fashion mood",
    "studio_black": "deep black seamless studio background, dramatic rim lighting, "
        "low-key moody fashion editorial, contrasted shadows",

    # === Intérieur lifestyle (intime, vie quotidienne) ===
    "loft": "modern minimalist loft, large industrial windows, morning sunshine streaming in, "
        "wooden floors, lifestyle photography",
    "bedroom_morning": "modern bedroom in morning light, white linen sheets gently rumpled, "
        "soft natural light through sheer curtains, cozy intimate atmosphere, lifestyle editorial",
    "bathroom_modern": "modern minimalist bathroom, marble surfaces, large mirror, "
        "soft natural light from window, post-shower steam, clean luxurious aesthetic",
    "kitchen_morning": "modern minimalist kitchen, morning sunshine, "
        "coffee mug on marble counter, casual home lifestyle",
    "lounge_penthouse": "luxury penthouse lounge, leather sofa, floor-to-ceiling windows, "
        "city skyline view at sunset, sophisticated atmosphere",
    "cabin_fireplace": "cozy mountain cabin interior, stone fireplace with crackling fire, "
        "knit throw blankets, warm autumn light, intimate cozy mood",

    # === Plein air nature ===
    "outdoor_beach": "tropical beach at golden hour, turquoise water, palm trees in background, "
        "wet sand reflecting sunset, warm golden light, summer vibes",
    "outdoor_beach_walking": "walking on tropical beach at golden hour, mid-stride pose, "
        "palm trees silhouette, turquoise ocean, soft waves on sand, "
        "warm sunset backlight, hair gently moving in sea breeze, cinematic editorial",
    "pool": "luxury pool deck at golden hour, turquoise water reflections, "
        "modern minimalist architecture, palm trees, warm summer light",
    "forest_morning": "dense pine forest in morning fog, soft diffused light through tall trees, "
        "moss-covered ground, mystical atmosphere, nature editorial",
    "mountain_vista": "mountain peak at sunrise, snow-capped peaks in distance, "
        "alpine landscape, dramatic wide vista, soft golden light",
    "lake_jetty": "wooden jetty over still mountain lake, mist rising from water, "
        "soft morning light, serene nature setting",
    "desert_dunes": "vast desert sand dunes at sunset, warm orange light, "
        "long dramatic shadows, cinematic landscape",

    # === Urbain ===
    "urban": "urban rooftop at sunset, modern architecture background, city skyline, "
        "cinematic warm light, fashion editorial",
    "urban_alley": "narrow urban alley with brick walls, soft evening light, "
        "discrete graffiti accents, gritty fashion editorial mood",
    "industrial_warehouse": "industrial warehouse loft, exposed brick walls, "
        "large windows with afternoon sun, raw concrete floors, fashion editorial",
    "balcony_city": "modern balcony with city skyline view at sunset, glass railings, "
        "golden hour reflections on glass towers",
    "subway_station": "modern subway station with geometric tile patterns, "
        "soft fluorescent lighting, clean urban editorial",
    "parking_garage": "modern concrete parking garage at dusk, dramatic side lighting, "
        "geometric columns, fashion editorial atmosphere",

    # === Sport / hôtel / luxe ===
    "gym_modern": "modern boutique gym with industrial design, large windows with morning sun, "
        "dumbbells and equipment in soft focus background, athletic editorial",
    "hotel_suite": "luxury hotel suite, large windows with ocean view, marble floors, "
        "white bed sheets, soft afternoon light, sophisticated travel editorial",
    "yacht_deck": "luxury yacht deck on calm Mediterranean sea, wooden planks, "
        "white sails, sunny afternoon, blue ocean horizon, summer travel",
    "spa_modern": "modern spa interior, dark wood and natural stone, "
        "soft warm candle light, serene wellness atmosphere",
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
