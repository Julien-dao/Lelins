from app.brands.base import BrandProfile

LELINS = BrandProfile(
    slug="lelins",
    name="Lelins",
    tagline="Le confort masculin, sans compromis.",
    mission=(
        "Habiller les hommes au quotidien avec des sous-vetements et vetements "
        "premium, durables et accessibles, vendus 100% en ligne."
    ),
    audience=(
        "Hommes 25-45 ans, urbains, soucieux de leur style et du confort, "
        "qui achetent en ligne et apprecient les marques DTC modernes."
    ),
    tone="direct, masculin, premium accessible, chaleureux, sans macho cliche",
    do=[
        "confort", "matieres", "coupe", "essentiels", "qualite",
        "durabilite", "made for men", "quotidien",
    ],
    dont=[
        "vulgarite", "sexisme", "promesses irrealistes", "jargon mode complique",
        "ton condescendant",
    ],
    languages=["fr"],
    currency="EUR",
    site_url=None,
)
