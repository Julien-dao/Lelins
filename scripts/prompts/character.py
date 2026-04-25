"""Description fine d'un mannequin (Character) pour Lelins.

Un Character regroupe ~40 paramètres décrivant un humain photoréaliste : âge,
morphologie, peau, cheveux, visage, corps, tatouages, piercings, marques
distinctives. À partir de ces paramètres on construit un fragment de prompt
SDXL réutilisable dans tous les rendus produit.

Usage typique :

    from prompts.character import Character

    c = Character.from_json_file("characters/lelins-main.json")
    head = c.build_prompt()             # description complète
    portrait = c.build_portrait_prompt()  # cadrage portrait/headshot
"""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Optional


# ---- Vocabulaires fermés (pour validation et auto-complétion) ----

ETHNICITIES = (
    "european", "mediterranean", "nordic", "slavic",
    "north_african", "middle_eastern",
    "sub_saharan_african", "black", "afro_caribbean",
    "east_asian", "south_asian", "southeast_asian",
    "latin_american", "hispanic",
    "mixed",
)

BODY_TYPES = ("slim", "lean", "athletic", "muscular", "stocky", "dad_bod", "mature_fit")
HEIGHTS = ("short", "average", "tall")

SKIN_TONES = (
    "very_pale", "pale", "fair", "light_olive", "olive",
    "tan", "light_brown", "brown", "dark_brown", "deep",
)
SKIN_TEXTURES = ("smooth", "light_freckles", "freckled", "weathered", "rough", "porous")
BODY_HAIR = ("none", "very_light", "light_chest", "hairy_chest", "hairy_chest_arms", "fully_hairy")

HAIR_COLORS = (
    "jet_black", "dark_brown", "brown", "light_brown", "auburn",
    "dark_blonde", "blonde", "platinum_blonde", "ginger", "red",
    "salt_pepper", "grey", "white", "dyed_silver", "dyed_color",
)
HAIR_LENGTHS = ("shaved", "buzz_cut", "short", "medium", "long", "very_long")
HAIR_STYLES = (
    "classic_short", "side_part", "slicked_back", "messy", "textured_crop",
    "curly", "wavy", "afro", "dreadlocks", "braids", "man_bun",
    "undercut", "fade", "mohawk",
)
HAIRLINES = ("full", "slight_recession", "receding", "balding_top", "fully_bald")

FACIAL_HAIR_STYLES = (
    "clean_shaven", "5_o_clock_shadow", "stubble", "short_beard",
    "full_beard", "long_beard", "goatee", "moustache_only",
    "sideburns_only", "mutton_chops", "anchor",
)
FACIAL_HAIR_DENSITY = ("none", "thin", "medium", "thick", "dense")

EYE_COLORS = ("brown", "dark_brown", "hazel", "green", "blue", "light_blue", "grey", "amber")
EYE_SHAPES = ("almond", "round", "hooded", "monolid", "deep_set", "downturned", "upturned")
EYEBROW_STYLES = ("thin", "medium", "thick", "bushy", "arched", "straight", "expressive")

NOSE_SHAPES = ("straight", "roman", "aquiline", "button", "wide", "narrow", "upturned", "hooked")
NOSE_SIZES = ("small", "medium", "large")

LIP_SHAPES = ("thin", "medium", "full", "heart_shaped", "asymmetric")
MOUTH_EXPRESSIONS = ("neutral", "slight_smile", "smile", "smirk", "serious", "intense")

JAW_SHAPES = ("square", "round", "narrow", "oval", "pointed", "wide")
CHINS = ("strong", "soft", "cleft", "dimpled", "recessed", "long")
CHEEKBONES = ("high", "prominent", "soft", "hollow", "medium")

SHOULDERS = ("narrow", "medium", "broad", "very_broad")
CHEST = ("flat", "lean", "defined", "muscular", "broad")
ARMS = ("slender", "toned", "muscular", "very_muscular")
HANDS = ("slender", "average", "large_strong", "veiny")
LEGS = ("slim", "toned", "muscular", "thick")

TATTOO_AESTHETICS = (
    "none", "fine_line", "minimalist", "blackwork",
    "traditional_american", "japanese_irezumi", "geometric",
    "tribal", "realistic", "watercolor", "lettering",
)

GLASSES = ("none", "round_metal", "square_acetate", "aviator", "wayfarer", "reading_thin")

EXPRESSIONS = (
    "confident", "neutral", "approachable", "smiling", "intense_gaze",
    "thoughtful", "laughing", "serious", "relaxed",
)


# ---- Mappings vers descripteurs anglais SDXL ----
# (les valeurs des champs sont des slugs ; on les traduit en phrases pour le prompt)

_ETHNICITY_DESC = {
    "european": "European",
    "mediterranean": "Mediterranean European",
    "nordic": "Nordic",
    "slavic": "Slavic",
    "north_african": "North African",
    "middle_eastern": "Middle Eastern",
    "sub_saharan_african": "Sub-Saharan African",
    "black": "Black",
    "afro_caribbean": "Afro-Caribbean",
    "east_asian": "East Asian",
    "south_asian": "South Asian",
    "southeast_asian": "Southeast Asian",
    "latin_american": "Latin American",
    "hispanic": "Hispanic",
    "mixed": "mixed-heritage",
}

_BODY_TYPE_DESC = {
    "slim": "slim build",
    "lean": "lean build",
    "athletic": "athletic build",
    "muscular": "muscular build",
    "stocky": "stocky build",
    "dad_bod": "soft dad-bod build",
    "mature_fit": "fit mature build",
}

_SKIN_TONE_DESC = {
    "very_pale": "very pale skin",
    "pale": "pale skin",
    "fair": "fair skin",
    "light_olive": "light olive skin",
    "olive": "olive skin",
    "tan": "tanned skin",
    "light_brown": "light brown skin",
    "brown": "brown skin",
    "dark_brown": "dark brown skin",
    "deep": "deep brown skin",
}

_SKIN_TEXTURE_DESC = {
    "smooth": "smooth skin",
    "light_freckles": "light freckles across the nose",
    "freckled": "freckled skin",
    "weathered": "weathered skin",
    "rough": "slightly rough skin texture",
    "porous": "visible skin pores, realistic texture",
}

_BODY_HAIR_DESC = {
    "none": "smooth hairless body",
    "very_light": "very light body hair",
    "light_chest": "light chest hair",
    "hairy_chest": "hairy chest",
    "hairy_chest_arms": "hairy chest and forearms",
    "fully_hairy": "fully hairy body",
}

_HAIR_COLOR_DESC = {
    "jet_black": "jet black", "dark_brown": "dark brown", "brown": "brown",
    "light_brown": "light brown", "auburn": "auburn",
    "dark_blonde": "dark blonde", "blonde": "blonde", "platinum_blonde": "platinum blonde",
    "ginger": "ginger", "red": "red",
    "salt_pepper": "salt-and-pepper", "grey": "grey", "white": "white",
    "dyed_silver": "dyed silver", "dyed_color": "vibrantly dyed",
}

_HAIR_LENGTH_DESC = {
    "shaved": "shaved", "buzz_cut": "buzz cut", "short": "short",
    "medium": "medium length", "long": "long", "very_long": "very long",
}

_HAIR_STYLE_DESC = {
    "classic_short": "classic short cut", "side_part": "side part",
    "slicked_back": "slicked back", "messy": "messy", "textured_crop": "textured crop",
    "curly": "curly", "wavy": "wavy", "afro": "afro",
    "dreadlocks": "dreadlocks", "braids": "braids", "man_bun": "man bun",
    "undercut": "undercut", "fade": "fade", "mohawk": "mohawk",
}

_HAIRLINE_DESC = {
    "full": "full hairline", "slight_recession": "slight hairline recession",
    "receding": "receding hairline", "balding_top": "balding crown",
    "fully_bald": "fully bald",
}

_FACIAL_HAIR_DESC = {
    "clean_shaven": "clean shaven",
    "5_o_clock_shadow": "five o'clock shadow",
    "stubble": "short stubble",
    "short_beard": "short beard",
    "full_beard": "full beard",
    "long_beard": "long beard",
    "goatee": "goatee",
    "moustache_only": "moustache",
    "sideburns_only": "prominent sideburns",
    "mutton_chops": "mutton chops",
    "anchor": "anchor beard",
}

_FACIAL_HAIR_DENSITY_DESC = {
    "none": "", "thin": "thin", "medium": "medium",
    "thick": "thick", "dense": "dense",
}

_EYE_COLOR_DESC = {
    "brown": "brown", "dark_brown": "dark brown", "hazel": "hazel",
    "green": "green", "blue": "blue", "light_blue": "light blue",
    "grey": "grey", "amber": "amber",
}
_EYE_SHAPE_DESC = {
    "almond": "almond-shaped", "round": "round", "hooded": "hooded",
    "monolid": "monolid", "deep_set": "deep set",
    "downturned": "downturned", "upturned": "upturned",
}
_EYEBROW_DESC = {
    "thin": "thin eyebrows", "medium": "medium eyebrows", "thick": "thick eyebrows",
    "bushy": "bushy eyebrows", "arched": "arched eyebrows",
    "straight": "straight eyebrows", "expressive": "expressive eyebrows",
}

_NOSE_SHAPE_DESC = {
    "straight": "straight", "roman": "Roman", "aquiline": "aquiline",
    "button": "button", "wide": "wide", "narrow": "narrow",
    "upturned": "slightly upturned", "hooked": "hooked",
}
_NOSE_SIZE_DESC = {"small": "small", "medium": "medium", "large": "large"}

_LIP_SHAPE_DESC = {
    "thin": "thin lips", "medium": "medium lips", "full": "full lips",
    "heart_shaped": "heart-shaped lips", "asymmetric": "asymmetric lips",
}
_MOUTH_EXPR_DESC = {
    "neutral": "neutral mouth", "slight_smile": "slight smile",
    "smile": "warm smile", "smirk": "subtle smirk",
    "serious": "serious expression", "intense": "intense expression",
}

_JAW_DESC = {
    "square": "square jaw", "round": "round jaw", "narrow": "narrow jaw",
    "oval": "oval jaw", "pointed": "pointed jaw", "wide": "wide jaw",
}
_CHIN_DESC = {
    "strong": "strong chin", "soft": "soft chin", "cleft": "cleft chin",
    "dimpled": "dimpled chin", "recessed": "recessed chin", "long": "long chin",
}
_CHEEKBONE_DESC = {
    "high": "high cheekbones", "prominent": "prominent cheekbones",
    "soft": "soft cheekbones", "hollow": "hollow cheeks", "medium": "medium cheekbones",
}

_SHOULDERS_DESC = {
    "narrow": "narrow shoulders", "medium": "medium shoulders",
    "broad": "broad shoulders", "very_broad": "very broad shoulders",
}
_CHEST_DESC = {
    "flat": "flat chest", "lean": "lean chest", "defined": "defined pectorals",
    "muscular": "muscular chest", "broad": "broad chest",
}
_ARMS_DESC = {
    "slender": "slender arms", "toned": "toned arms",
    "muscular": "muscular arms", "very_muscular": "very muscular arms",
}
_HANDS_DESC = {
    "slender": "slender hands", "average": "average hands",
    "large_strong": "large strong hands", "veiny": "veiny hands",
}
_LEGS_DESC = {
    "slim": "slim legs", "toned": "toned legs",
    "muscular": "muscular legs", "thick": "thick muscular legs",
}

_TATTOO_AESTHETIC_DESC = {
    "fine_line": "fine line", "minimalist": "minimalist",
    "blackwork": "blackwork", "traditional_american": "American traditional",
    "japanese_irezumi": "Japanese irezumi", "geometric": "geometric",
    "tribal": "tribal", "realistic": "realistic",
    "watercolor": "watercolor", "lettering": "lettering",
}

_GLASSES_DESC = {
    "none": "",
    "round_metal": "wearing round metal-frame glasses",
    "square_acetate": "wearing square acetate glasses",
    "aviator": "wearing aviator glasses",
    "wayfarer": "wearing Wayfarer-style glasses",
    "reading_thin": "wearing thin reading glasses",
}

_EXPRESSION_DESC = {
    "confident": "confident expression", "neutral": "neutral expression",
    "approachable": "approachable expression", "smiling": "natural smile",
    "intense_gaze": "intense gaze", "thoughtful": "thoughtful expression",
    "laughing": "natural laugh", "serious": "serious expression",
    "relaxed": "relaxed expression",
}


# ---- Le dataclass ----

@dataclass
class Character:
    """Profil mannequin Lelins. Sérialisable en JSON pour réutilisation."""

    # Identité
    name: str
    seed: Optional[int] = None  # rempli après la première génération réussie
    portrait_path: Optional[str] = None  # rempli par generate_character.py — utilisé comme référence IP-Adapter
    age: int = 28

    # 40 paramètres ↓
    ethnicity: str = "european"               # 1
    body_type: str = "athletic"               # 2
    height: str = "average"                   # 3

    skin_tone: str = "light_olive"            # 4
    skin_texture: str = "smooth"              # 5
    skin_marks: list[str] = field(default_factory=list)  # 6 (free-text list)
    body_hair: str = "light_chest"            # 7

    hair_color: str = "dark_brown"            # 8
    hair_length: str = "short"                # 9
    hair_style: str = "classic_short"         # 10
    hairline: str = "full"                    # 11

    facial_hair_style: str = "stubble"        # 12
    facial_hair_density: str = "medium"       # 13

    eye_color: str = "brown"                  # 14
    eye_shape: str = "almond"                 # 15
    eyebrow_style: str = "medium"             # 16

    nose_shape: str = "straight"              # 17
    nose_size: str = "medium"                 # 18

    lip_shape: str = "medium"                 # 19
    mouth_expression: str = "neutral"         # 20

    jaw_shape: str = "square"                 # 21
    chin: str = "strong"                      # 22
    cheekbones: str = "medium"                # 23

    shoulders: str = "medium"                 # 24
    chest: str = "defined"                    # 25
    arms: str = "toned"                       # 26
    hands: str = "average"                    # 27
    legs: str = "toned"                       # 28

    tattoo_zones: list[str] = field(default_factory=list)  # 29 (free-text: "left arm half sleeve")
    tattoo_aesthetic: str = "fine_line"       # 30 (utilisé seulement si tattoo_zones non vide)

    piercings: list[str] = field(default_factory=list)  # 31 (free-text: "small left ear stud")

    glasses: str = "none"                     # 32

    distinguishing_marks: list[str] = field(default_factory=list)  # 33 (cicatrices, naevus, vitiligo, etc.)

    expression: str = "confident"             # 34

    # Champs annexes utiles pour la cohérence
    notes: str = ""                           # 35 (texte libre additionnel injecté en fin de prompt)
    style_keywords: list[str] = field(default_factory=list)  # 36 (mots-clés esthétiques additionnels)

    # ---- Construction du prompt ----

    def build_prompt(self) -> str:
        """Construit le fragment de prompt humain. Ne contient ni vêtement ni décor."""
        parts: list[str] = []

        # Phrase d'ouverture : âge + ethnicité + morpho
        ethn = _ETHNICITY_DESC.get(self.ethnicity, self.ethnicity)
        bt = _BODY_TYPE_DESC.get(self.body_type, self.body_type)
        parts.append(f"{self.age} year old {ethn} adult man, {bt}")

        if self.height != "average":
            parts.append(f"{self.height} stature")

        # Peau
        parts.append(_SKIN_TONE_DESC.get(self.skin_tone, self.skin_tone))
        if self.skin_texture != "smooth":
            parts.append(_SKIN_TEXTURE_DESC.get(self.skin_texture, self.skin_texture))
        for mark in self.skin_marks:
            parts.append(mark)

        # Pilosité corporelle
        if self.body_hair != "none":
            parts.append(_BODY_HAIR_DESC.get(self.body_hair, self.body_hair))

        # Cheveux
        hair_color = _HAIR_COLOR_DESC.get(self.hair_color, self.hair_color)
        hair_length = _HAIR_LENGTH_DESC.get(self.hair_length, self.hair_length)
        hair_style = _HAIR_STYLE_DESC.get(self.hair_style, self.hair_style)
        if self.hairline == "fully_bald":
            parts.append("fully bald head")
        else:
            parts.append(f"{hair_color} {hair_length} {hair_style} hair")
            if self.hairline not in ("full",):
                parts.append(_HAIRLINE_DESC.get(self.hairline, self.hairline))

        # Pilosité faciale
        if self.facial_hair_style != "clean_shaven":
            density = _FACIAL_HAIR_DENSITY_DESC.get(self.facial_hair_density, "")
            style = _FACIAL_HAIR_DESC.get(self.facial_hair_style, self.facial_hair_style)
            parts.append(f"{density} {style}".strip())
        else:
            parts.append("clean shaven")

        # Yeux
        eye_color = _EYE_COLOR_DESC.get(self.eye_color, self.eye_color)
        eye_shape = _EYE_SHAPE_DESC.get(self.eye_shape, self.eye_shape)
        parts.append(f"{eye_color} {eye_shape} eyes")
        parts.append(_EYEBROW_DESC.get(self.eyebrow_style, self.eyebrow_style))

        # Nez
        nose_shape = _NOSE_SHAPE_DESC.get(self.nose_shape, self.nose_shape)
        nose_size = _NOSE_SIZE_DESC.get(self.nose_size, self.nose_size)
        parts.append(f"{nose_size} {nose_shape} nose")

        # Bouche
        parts.append(_LIP_SHAPE_DESC.get(self.lip_shape, self.lip_shape))
        parts.append(_MOUTH_EXPR_DESC.get(self.mouth_expression, self.mouth_expression))

        # Mâchoire / menton / pommettes
        parts.append(_JAW_DESC.get(self.jaw_shape, self.jaw_shape))
        parts.append(_CHIN_DESC.get(self.chin, self.chin))
        parts.append(_CHEEKBONE_DESC.get(self.cheekbones, self.cheekbones))

        # Corps
        parts.append(_SHOULDERS_DESC.get(self.shoulders, self.shoulders))
        parts.append(_CHEST_DESC.get(self.chest, self.chest))
        parts.append(_ARMS_DESC.get(self.arms, self.arms))
        parts.append(_HANDS_DESC.get(self.hands, self.hands))
        parts.append(_LEGS_DESC.get(self.legs, self.legs))

        # Tatouages
        if self.tattoo_zones:
            aesthetic = _TATTOO_AESTHETIC_DESC.get(self.tattoo_aesthetic, self.tattoo_aesthetic)
            zones = ", ".join(self.tattoo_zones)
            parts.append(f"{aesthetic} tattoos on {zones}")

        # Piercings
        for p in self.piercings:
            parts.append(p)

        # Lunettes
        glasses = _GLASSES_DESC.get(self.glasses, "")
        if glasses:
            parts.append(glasses)

        # Marques distinctives
        for m in self.distinguishing_marks:
            parts.append(m)

        # Expression
        parts.append(_EXPRESSION_DESC.get(self.expression, self.expression))

        # Mots-clés esthétiques additionnels
        for kw in self.style_keywords:
            parts.append(kw)

        # Notes libres
        if self.notes:
            parts.append(self.notes)

        return ", ".join(p for p in parts if p)

    def build_portrait_prompt(self) -> str:
        """Variante : portrait/headshot pour valider l'identité du mannequin."""
        base = self.build_prompt()
        framing = (
            "professional studio headshot, head and shoulders portrait, "
            "centered composition, neutral grey background, soft beauty lighting, "
            "shallow depth of field, photorealistic, 8k, shot on Canon EOS R5, 85mm lens"
        )
        return f"{base}, {framing}"

    # ---- (dé)sérialisation JSON ----

    def to_json(self, path: Path) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(
            json.dumps(asdict(self), indent=2, ensure_ascii=False),
            encoding="utf-8",
        )

    @classmethod
    def from_json_file(cls, path: Path | str) -> "Character":
        data = json.loads(Path(path).read_text(encoding="utf-8"))
        # On filtre aux seuls champs connus pour rester tolérant aux extensions futures
        known = {f for f in cls.__dataclass_fields__}
        return cls(**{k: v for k, v in data.items() if k in known})

    @classmethod
    def from_dict(cls, data: dict) -> "Character":
        known = {f for f in cls.__dataclass_fields__}
        return cls(**{k: v for k, v in data.items() if k in known})
