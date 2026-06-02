from pydantic import BaseModel, Field


class BrandProfile(BaseModel):
    slug: str
    name: str
    tagline: str
    mission: str
    audience: str
    tone: str = Field(description="Ton de marque, ex: 'direct, masculin, premium, decontracte'")
    do: list[str] = Field(default_factory=list, description="Mots / themes a privilegier")
    dont: list[str] = Field(default_factory=list, description="Mots / themes a eviter")
    languages: list[str] = Field(default_factory=lambda: ["fr"])
    currency: str = "EUR"
    site_url: str | None = None

    def to_prompt_block(self) -> str:
        do_str = ", ".join(self.do) or "(aucun)"
        dont_str = ", ".join(self.dont) or "(aucun)"
        return (
            f"Marque: {self.name}\n"
            f"Slogan: {self.tagline}\n"
            f"Mission: {self.mission}\n"
            f"Cible: {self.audience}\n"
            f"Ton de marque: {self.tone}\n"
            f"A privilegier: {do_str}\n"
            f"A eviter: {dont_str}\n"
            f"Langues: {', '.join(self.languages)}\n"
            f"Devise: {self.currency}\n"
        )
