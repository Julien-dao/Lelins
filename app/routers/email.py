from fastapi import APIRouter, Form, Request
from fastapi.responses import HTMLResponse

from app.agents.email import EMAIL_TYPES, EmailAgent, EmailBrief
from app.routers._helpers import base_context, format_error, resolve_brand, templates
from app.web_import import ImportError_, import_product, imported_from_form

router = APIRouter(prefix="/email", tags=["email"])


def _trim(text: str, limit: int = 400) -> str:
    text = (text or "").strip()
    return text if len(text) <= limit else text[: limit - 1].rstrip() + "..."


@router.get("", response_class=HTMLResponse)
async def form(request: Request, brand: str | None = None) -> HTMLResponse:
    return templates.TemplateResponse(
        request,
        "email.html",
        {
            **base_context(request, brand),
            "result": None,
            "error": None,
            "form_data": {},
            "email_types": EMAIL_TYPES,
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
            offer_line = imported.title
            if imported.price:
                offer_line = f"{imported.title} ({imported.price})"
            form_data = {
                "email_type": "promo",
                "product_or_offer": offer_line,
                "customer_segment": "",
                "extra_context": _trim(imported.description),
            }
            brand_profile = resolve_brand(brand or None)
            agent = EmailAgent(brand=brand_profile)
            result = await agent.write_email(EmailBrief(**form_data))
    except ImportError_ as exc:
        error = str(exc)
    except Exception as exc:
        error = format_error(exc)

    return templates.TemplateResponse(
        request,
        "email.html",
        {
            **base_context(request, brand or None),
            "result": result,
            "error": error,
            "form_data": form_data,
            "email_types": EMAIL_TYPES,
            "imported": imported,
        },
    )


@router.post("", response_class=HTMLResponse)
async def submit(
    request: Request,
    brand: str = Form(default=""),
    email_type: str = Form(...),
    product_or_offer: str = Form(default=""),
    customer_segment: str = Form(default=""),
    extra_context: str = Form(default=""),
    imported_image_url: str = Form(default=""),
    imported_source_url: str = Form(default=""),
    imported_title: str = Form(default=""),
    imported_price: str = Form(default=""),
) -> HTMLResponse:
    form_data = {
        "email_type": email_type,
        "product_or_offer": product_or_offer,
        "customer_segment": customer_segment,
        "extra_context": extra_context,
    }
    result, error = None, None
    try:
        brand_profile = resolve_brand(brand or None)
        agent = EmailAgent(brand=brand_profile)
        result = await agent.write_email(EmailBrief(**form_data))
    except Exception as exc:
        error = format_error(exc)

    return templates.TemplateResponse(
        request,
        "email.html",
        {
            **base_context(request, brand or None),
            "result": result,
            "error": error,
            "form_data": form_data,
            "email_types": EMAIL_TYPES,
            "imported": imported_from_form(
                image_url=imported_image_url,
                source_url=imported_source_url,
                title=imported_title,
                price=imported_price,
            ),
        },
    )
