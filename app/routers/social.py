from fastapi import APIRouter, Form, Request
from fastapi.responses import HTMLResponse

from app.agents.social import PLATFORMS, SocialAgent, SocialBrief
from app.routers._helpers import base_context, format_error, resolve_brand, templates

router = APIRouter(prefix="/social", tags=["social"])


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
        brief = SocialBrief(**form_data)
        result = await agent.write_posts(brief)
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
        },
    )
