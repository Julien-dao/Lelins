from pydantic import BaseModel, Field

from app.agents.base import BaseAgent


class ProductBrief(BaseModel):
    product_name: str
    category: str = Field(default="", description="Ex: boxer, t-shirt, chaussette")
    materials: str = Field(default="", description="Ex: 95% coton bio, 5% elasthanne")
    sizes: str = Field(default="", description="Ex: S, M, L, XL")
    colors: str = Field(default="", description="Ex: noir, blanc, marine")
    price: str = Field(default="", description="Ex: 19,90 EUR")
    key_benefits: str = Field(default="", description="Liste libre, points forts")
    target_keyword: str = Field(default="", description="Mot-cle SEO principal")


class CopywriterAgent(BaseAgent):
    role = "Copywriter e-commerce"
    job_description = (
        "Tu rediges des fiches produits qui convertissent : titre SEO, "
        "description courte (meta), description longue avec bullet points, "
        "et un encart objections / FAQ courte. Tu n'inventes pas de "
        "caracteristiques techniques absentes du brief : si une info manque, "
        "tu poses une question ou tu utilises une formulation generique."
    )

    def build_prompt(self, brief: ProductBrief) -> str:
        return (
            "Redige une fiche produit complete pour ce produit physique.\n\n"
            f"Brief produit :\n"
            f"- Nom : {brief.product_name}\n"
            f"- Categorie : {brief.category or 'non precise'}\n"
            f"- Matieres : {brief.materials or 'non precisees'}\n"
            f"- Tailles : {brief.sizes or 'non precisees'}\n"
            f"- Couleurs : {brief.colors or 'non precisees'}\n"
            f"- Prix : {brief.price or 'non precise'}\n"
            f"- Benefices cles : {brief.key_benefits or 'a deduire du produit'}\n"
            f"- Mot-cle SEO cible : {brief.target_keyword or '(libre)'}\n\n"
            "Format attendu (Markdown) :\n"
            "## Titre SEO (max 60 car.)\n"
            "## Meta description (max 155 car.)\n"
            "## Description longue\n"
            "## Bullet points (5)\n"
            "## Mini-FAQ (3 Q/R)\n"
            "## Suggestions de tags / categories\n"
        )

    async def write_product_page(self, brief: ProductBrief) -> str:
        return await self.run(self.build_prompt(brief), temperature=0.6)
