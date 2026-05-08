from pydantic import BaseModel

from app.agents.base import BaseAgent

PLATFORMS = {
    "instagram": "Instagram (post feed + caption + 5 a 10 hashtags pertinents)",
    "tiktok": "TikTok (script de reel/short de 20-30s, hook fort, structure secondee)",
    "facebook": "Facebook (post feed, ton communautaire, longueur moyenne)",
    "linkedin": "LinkedIn (angle marque / coulisses business, ton plus pro)",
}


class SocialBrief(BaseModel):
    platform: str
    topic: str
    product_or_collection: str = ""
    angle: str = ""
    n_variants: int = 3


class SocialAgent(BaseAgent):
    role = "Social media manager e-commerce"
    job_description = (
        "Tu produis des posts pour reseaux sociaux qui drive du trafic et de "
        "l'engagement vers la boutique. Tu adaptes naturellement le format "
        "et le ton a chaque plateforme. Tu proposes plusieurs variantes "
        "differenciees (pas juste reformulees)."
    )

    def build_prompt(self, brief: SocialBrief) -> str:
        platform_desc = PLATFORMS.get(brief.platform, brief.platform)
        n = max(1, min(brief.n_variants, 6))
        return (
            f"Plateforme : {platform_desc}\n"
            f"Sujet / theme : {brief.topic}\n"
            f"Produit ou collection : {brief.product_or_collection or 'aucun specifique'}\n"
            f"Angle souhaite : {brief.angle or 'libre'}\n"
            f"Nombre de variantes : {n}\n\n"
            "Format attendu (Markdown) :\n"
            f"## Variante 1 ... ## Variante {n}, chacune avec :\n"
            "- Hook / accroche\n"
            "- Corps du post (ou script video si TikTok)\n"
            "- Call-to-action\n"
            "- Hashtags (si applicable)\n"
            "## Idees visuelles (1 ligne par variante)\n"
            "## Meilleur creneau de publication suggere\n"
        )

    async def write_posts(self, brief: SocialBrief) -> str:
        return await self.run(self.build_prompt(brief), temperature=0.85)
