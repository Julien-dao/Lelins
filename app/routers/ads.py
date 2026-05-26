from fastapi import APIRouter, Form, Request
from fastapi.responses import HTMLResponse

from app.agents.ads import AD_PLATFORMS, AdBrief, AdsAgent
from app.routers._helpers import base_context, format_error, resolve_brand, templates

router = APIRouter(prefix="/ads", tags=["ads"])


@router.get("", response_class=HTMLResponse)
async def form(request: Request, brand: str | None = None) -> HTMLResponse:
    return templates.TemplateResponse(
        request,
        "ads.html",
        {
            **base_context(request, brand),
            "result": None,
            "error": None,
            "form_data": {},
            "ad_platforms": AD_PLATFORMS,
        },
    )


@router.post("", response_class=HTMLResponse)
async def submit(
    request: Request,
    brand: str = Form(default=""),
    platform: str = Form(...),
    product_or_offer: str = Form(...),
    target_persona: str = Form(default=""),
    pain_or_desire: str = Form(default=""),
    promo: str = Form(default=""),
    n_variants: int = Form(default=4),
) -> HTMLResponse:
    form_data = {
        "platform": platform,
        "product_or_offer": product_or_offer,
        "target_persona": target_persona,
        "pain_or_desire": pain_or_desire,
        "promo": promo,
        "n_variants": n_variants,
    }
    result, error = None, None
    try:
        brand_profile = resolve_brand(brand or None)
        agent = AdsAgent(brand=brand_profile)
        brief = AdBrief(**form_data)
        result = await agent.write_ads(brief)
    except Exception as exc:
        error = format_error(exc)

    return templates.TemplateResponse(
        request,
        "ads.html",
        {
            **base_context(request, brand or None),
            "result": result,
            "error": error,
            "form_data": form_data,
            "ad_platforms": AD_PLATFORMS,
        },
    )
