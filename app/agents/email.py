from pydantic import BaseModel

from app.agents.base import BaseAgent

EMAIL_TYPES = {
    "welcome": "Email de bienvenue (premier achat / nouvelle inscription newsletter).",
    "abandoned_cart": "Relance panier abandonne (1er rappel, ton amical).",
    "abandoned_cart_2": "2eme relance panier abandonne avec un leger incentive (livraison ou code).",
    "post_purchase": "Email post-achat : remerciement, conseils d'entretien, cross-sell discret.",
    "winback": "Reactivation client inactif depuis plus de 90 jours.",
    "promo": "Annonce d'une promotion / drop produit, ton hype mais sobre.",
}


class EmailBrief(BaseModel):
    email_type: str
    product_or_offer: str = ""
    customer_segment: str = ""
    extra_context: str = ""


class EmailAgent(BaseAgent):
    role = "Responsable email marketing CRM"
    job_description = (
        "Tu rediges des emails marketing pour e-commerce DTC : objet, "
        "preheader, corps en HTML simple (titres, paragraphes courts, CTA), "
        "et une variante d'objet pour A/B test. Tu evites le spam-y "
        "(majuscules, points d'exclamation a repetition, mots black-list)."
    )

    def build_prompt(self, brief: EmailBrief) -> str:
        type_desc = EMAIL_TYPES.get(brief.email_type, brief.email_type)
        return (
            f"Type d'email : {brief.email_type} ({type_desc})\n"
            f"Produit ou offre mise en avant : {brief.product_or_offer or 'aucun specifique'}\n"
            f"Segment client : {brief.customer_segment or 'tout client'}\n"
            f"Contexte additionnel : {brief.extra_context or 'aucun'}\n\n"
            "Format attendu (Markdown) :\n"
            "## Objet (option A)\n"
            "## Objet (option B - test)\n"
            "## Preheader\n"
            "## Corps de l'email (en HTML simple, max 180 mots)\n"
            "## Bouton CTA (texte + suggestion de lien)\n"
            "## Notes anti-spam (1-2 lignes)\n"
        )

    async def write_email(self, brief: EmailBrief) -> str:
        return await self.run(self.build_prompt(brief), temperature=0.7)
