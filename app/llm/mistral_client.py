import httpx

from app.config import get_settings

MISTRAL_BASE_URL = "https://api.mistral.ai/v1"


class MistralError(RuntimeError):
    pass


class MistralClient:
    def __init__(self, api_key: str | None = None, model: str | None = None) -> None:
        settings = get_settings()
        self.api_key = api_key or settings.mistral_api_key
        self.model = model or settings.mistral_model

    async def chat(
        self,
        system: str,
        user: str,
        *,
        temperature: float = 0.7,
        max_tokens: int = 1200,
    ) -> str:
        if not self.api_key:
            raise MistralError(
                "MISTRAL_API_KEY manquante. Cree un compte gratuit sur "
                "https://console.mistral.ai/ puis renseigne la cle dans .env."
            )

        payload = {
            "model": self.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user},
            ],
            "temperature": temperature,
            "max_tokens": max_tokens,
        }
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

        async with httpx.AsyncClient(timeout=60.0) as client:
            resp = await client.post(
                f"{MISTRAL_BASE_URL}/chat/completions",
                json=payload,
                headers=headers,
            )

        if resp.status_code == 401:
            raise MistralError("Cle API Mistral invalide (401).")
        if resp.status_code == 429:
            raise MistralError("Quota free tier Mistral atteint, reessaie plus tard (429).")
        if resp.status_code >= 400:
            raise MistralError(f"Erreur Mistral {resp.status_code}: {resp.text}")

        data = resp.json()
        try:
            return data["choices"][0]["message"]["content"].strip()
        except (KeyError, IndexError) as exc:
            raise MistralError(f"Reponse Mistral inattendue: {data}") from exc
