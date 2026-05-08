from fastapi import APIRouter, Form, Request
from fastapi.responses import HTMLResponse

from app.agents.email import EMAIL_TYPES, EmailAgent, EmailBrief
from app.routers._helpers import base_context, format_error, resolve_brand, templates

router = APIRouter(prefix="/email", tags=["email"])


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
        brief = EmailBrief(**form_data)
        result = await agent.write_email(brief)
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
        },
    )
