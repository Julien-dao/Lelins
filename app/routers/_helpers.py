from pathlib import Path

from fastapi import Request
from fastapi.templating import Jinja2Templates

from app.brands import BRANDS, get_brand
from app.config import get_settings
from app.llm.mistral_client import MistralError

TEMPLATES_DIR = Path(__file__).resolve().parent.parent / "templates"
templates = Jinja2Templates(directory=str(TEMPLATES_DIR))


def base_context(request: Request, brand_slug: str | None = None) -> dict:
    settings = get_settings()
    slug = brand_slug or settings.default_brand
    return {
        "brands": list(BRANDS.values()),
        "active_brand": slug,
        "mistral_configured": bool(settings.mistral_api_key),
        "model": settings.mistral_model,
    }


def resolve_brand(slug: str | None):
    settings = get_settings()
    return get_brand(slug or settings.default_brand)


def format_error(exc: Exception) -> str:
    if isinstance(exc, MistralError):
        return str(exc)
    return f"Erreur inattendue : {exc}"
