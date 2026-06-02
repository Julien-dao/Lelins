from fastapi import APIRouter, Form, Request
from fastapi.responses import HTMLResponse

from app.agents.social import PLATFORMS, SocialAgent, SocialBrief
from app.routers._helpers import base_context, format_error, resolve_brand, templates
from app.web_import import ImportError_, import_product, imported_from_form

router = APIRouter(prefix="/social", tags=["social"])


def _trim(text: str, limit: int = 240) -> str:
    text = (text or "").strip()
    return text if len(text) <= limit else text[: limit - 1].rstrip() + "..."


@router.get("", response_class=HTMLResponse)
async def form(request: Request, brand: str | None = None) -> HTMLResponse:
    return templates.TemplateResponse(
        request,
        "social.html",
        {
            **base_context(request, brand),
            "result": None,
            "error": None,
            "form_data": {},
            "platforms": PLATFORMS,
            "imported": None,
        },
    )


@router.post("/import", response_class=HTMLResponse)
async def import_and_generate(
    request: Request,
    url: str = Form(...),
    brand: str = Form(default=""),
) -> HTMLResponse:
    result, error, imported = None, None, None
    form_data: dict = {}
    try:
        imported = await import_product(url)
        if imported.is_empty:
            error = (
                "Aucune info produit detectee sur cette page (pas de balises "
                "Open Graph ni de donnees structurees). Saisis le brief a la main."
            )
        else:
            form_data = {
                "platform": "instagram",
                "topic": imported.title or "Nouveau produit",
                "product_or_collection": imported.title,
                "angle": _trim(imported.description),
                "n_variants": 3,
            }
            brand_profile = resolve_brand(brand or None)
            agent = SocialAgent(brand=brand_profile)
            result = await agent.write_posts(SocialBrief(**form_data))
    except ImportError_ as exc:
        error = str(exc)
    except Exception as exc:
        error = format_error(exc)

    return templates.TemplateResponse(
        request,
        "social.html",
        {
            **base_context(request, brand or None),
            "result": result,
            "error": error,
            "form_data": form_data,
            "platforms": PLATFORMS,
            "imported": imported,
        },
    )


@router.post("", response_class=HTMLResponse)
async def submit(
    request: Request,
    brand: str = Form(default=""),
    platform: str = Form(...),
    topic: str = Form(...),
    product_or_collection: str = Form(default=""),
    angle: str = Form(default=""),
    n_variants: int = Form(default=3),
    imported_image_url: str = Form(default=""),
    imported_source_url: str = Form(default=""),
    imported_title: str = Form(default=""),
    imported_price: str = Form(default=""),
) -> HTMLResponse:
    form_data = {
        "platform": platform,
        "topic": topic,
        "product_or_collection": product_or_collection,
        "angle": angle,
        "n_variants": n_variants,
    }
    result, error = None, None
    try:
        brand_profile = resolve_brand(brand or None)
        agent = SocialAgent(brand=brand_profile)
        result = await agent.write_posts(SocialBrief(**form_data))
    except Exception as exc:
        error = format_error(exc)

    return templates.TemplateResponse(
        request,
        "social.html",
        {
            **base_context(request, brand or None),
            "result": result,
            "error": error,
            "form_data": form_data,
            "platforms": PLATFORMS,
            "imported": imported_from_form(
                image_url=imported_image_url,
                source_url=imported_source_url,
                title=imported_title,
                price=imported_price,
            ),
        },
    )
