from app.brands.base import BrandProfile
from app.brands.lelins import LELINS

BRANDS: dict[str, BrandProfile] = {
    LELINS.slug: LELINS,
}


def get_brand(slug: str) -> BrandProfile:
    if slug not in BRANDS:
        raise KeyError(f"Marque inconnue: {slug}. Disponibles: {list(BRANDS)}")
    return BRANDS[slug]
