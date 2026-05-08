from pydantic import BaseModel

from app.agents.base import BaseAgent

AD_PLATFORMS = {
    "meta": "Meta Ads (Facebook + Instagram, format feed + story).",
    "google_search": "Google Ads Search : 15 titres (max 30 car.) + 4 descriptions (max 90 car.).",
    "google_pmax": "Google Performance Max : assets titres courts/longs, descriptions, signaux audience.",
    "tiktok": "TikTok Ads : hooks de 3 secondes + script court 15-20s.",
}


class AdBrief(BaseModel):
    platform: str
    product_or_offer: str
    target_persona: str = ""
    pain_or_desire: str = ""
    promo: str = ""
    n_variants: int = 4


class AdsAgent(BaseAgent):
    role = "Media buyer / copywriter publicitaire"
    job_description = (
        "Tu rediges des creatives publicitaires pensees direct response : "
        "hook fort, promesse claire, preuve, CTA. Tu connais les contraintes "
        "de chaque plateforme (limites de caracteres Google Ads, regles Meta, "
        "format vertical TikTok). Tu evites les promesses interdites "
        "(sante, garantie absolue, etc.)."
    )

    def build_prompt(self, brief: AdBrief) -> str:
        platform_desc = AD_PLATFORMS.get(brief.platform, brief.platform)
        n = max(1, min(brief.n_variants, 8))
        return (
            f"Plateforme : {platform_desc}\n"
            f"Produit ou offre : {brief.product_or_offer}\n"
            f"Persona cible : {brief.target_persona or 'la cible naturelle de la marque'}\n"
            f"Douleur / desir adresse : {brief.pain_or_desire or 'a deduire'}\n"
            f"Promo ou incentive : {brief.promo or 'aucun'}\n"
            f"Nombre d'angles / variantes : {n}\n\n"
            "Format attendu (Markdown) :\n"
            f"## Variante 1 ... ## Variante {n}, avec pour chacune :\n"
            "- Angle / promesse\n"
            "- Hook (3 premieres secondes / premiere ligne)\n"
            "- Texte principal\n"
            "- Headline(s) si la plateforme le demande\n"
            "- Description(s) si la plateforme le demande\n"
            "- CTA\n"
            "## Recap audiences / mots-cles suggeres\n"
            "## Avertissement compliance (1 ligne)\n"
        )

    async def write_ads(self, brief: AdBrief) -> str:
        return await self.run(self.build_prompt(brief), temperature=0.8)
