from app.brands.base import BrandProfile
from app.llm.mistral_client import MistralClient


class BaseAgent:
    role: str = "Marketeur"
    job_description: str = ""

    def __init__(self, brand: BrandProfile, llm: MistralClient | None = None) -> None:
        self.brand = brand
        self.llm = llm or MistralClient()

    def system_prompt(self) -> str:
        return (
            f"Tu es {self.role} senior dans une agence de marketing direct "
            f"specialisee en e-commerce de produits physiques.\n"
            f"{self.job_description}\n\n"
            f"Voici la marque pour laquelle tu travailles :\n"
            f"{self.brand.to_prompt_block()}\n"
            f"Reponds toujours en francais, avec un ton coherent avec la marque, "
            f"et structure ta reponse en Markdown lisible."
        )

    async def run(self, user_prompt: str, *, temperature: float = 0.7) -> str:
        return await self.llm.chat(
            system=self.system_prompt(),
            user=user_prompt,
            temperature=temperature,
        )
