"""Studio visuel : generation de mannequin (text-to-image) + essayage virtuel.

La couche fournisseur est isolee ici. Aujourd'hui un seul fournisseur de
reference est implemente (fal.ai), choisi parce qu'une seule API couvre les
deux etapes. Pour en brancher un autre (Replicate, GPU auto-heberge...), il
suffit d'ajouter une branche dans `_run_model` et d'adapter les payloads.

Tant que `image_provider` / `image_api_key` ne sont pas configures, les
fonctions levent une `ImageStudioError` explicite : aucune image n'est
generee, mais rien ne casse.
"""

import base64

import httpx

from app.config import get_settings


class ImageStudioError(Exception):
    pass


def studio_configured() -> bool:
    settings = get_settings()
    return bool(settings.image_provider and settings.image_api_key)


def _require_config() -> None:
    if not studio_configured():
        raise ImageStudioError(
            "Studio visuel non configure. Choisis un fournisseur d'image "
            "(ex. fal.ai), cree une cle API, puis renseigne IMAGE_PROVIDER "
            "et IMAGE_API_KEY dans les secrets du Space."
        )


def to_data_uri(content: bytes, content_type: str) -> str:
    encoded = base64.b64encode(content).decode("ascii")
    return f"data:{content_type};base64,{encoded}"


async def _run_model(model_id: str, payload: dict) -> dict:
    """Appelle le fournisseur configure et renvoie la reponse JSON brute."""
    settings = get_settings()
    provider = settings.image_provider.lower()

    if provider == "fal":
        url = f"https://fal.run/{model_id}"
        headers = {"Authorization": f"Key {settings.image_api_key}"}
    else:
        raise ImageStudioError(
            f"Fournisseur d'image inconnu : '{settings.image_provider}'. "
            "Seul 'fal' est implemente pour l'instant."
        )

    try:
        async with httpx.AsyncClient(timeout=180.0) as client:
            response = await client.post(url, json=payload, headers=headers)
    except httpx.HTTPError as exc:
        raise ImageStudioError(f"Appel au fournisseur impossible : {exc}") from exc

    if response.status_code >= 400:
        raise ImageStudioError(
            f"Fournisseur image {response.status_code} : {response.text[:400]}"
        )
    return response.json()


def _first_image_url(data: dict) -> str:
    images = data.get("images")
    if isinstance(images, list) and images:
        first = images[0]
        if isinstance(first, dict) and first.get("url"):
            return first["url"]
        if isinstance(first, str):
            return first
    image = data.get("image")
    if isinstance(image, dict) and image.get("url"):
        return image["url"]
    raise ImageStudioError(
        "Le fournisseur n'a renvoye aucune image (contenu refuse par le "
        "filtre, ou format de reponse inattendu)."
    )


async def generate_mannequin(prompt: str) -> str:
    """Genere un corps de mannequin et renvoie l'URL de l'image."""
    _require_config()
    settings = get_settings()
    full_prompt = (
        f"{prompt.strip()}. Studio photo, realistic, full body, neutral pose, "
        "soft lighting, plain background, e-commerce model shot"
    )
    data = await _run_model(
        settings.image_gen_model,
        {"prompt": full_prompt, "image_size": "portrait_4_3", "num_images": 1},
    )
    return _first_image_url(data)


async def try_on(model_image: str, garment_data_uri: str, category: str) -> str:
    """Habille `model_image` (URL) avec le vetement (data URI). Renvoie l'URL."""
    _require_config()
    settings = get_settings()
    data = await _run_model(
        settings.image_tryon_model,
        {
            "model_image": model_image,
            "garment_image": garment_data_uri,
            "category": category or "bottoms",
        },
    )
    return _first_image_url(data)
