from fastapi import APIRouter, Form, Request
from fastapi.responses import HTMLResponse

from app.agents.copywriter import CopywriterAgent, ProductBrief
from app.routers._helpers import base_context, format_error, resolve_brand, templates
from app.web_import import ImportError_, import_product, imported_from_form

router = APIRouter(prefix="/copywriter", tags=["copywriter"])


@router.get("", response_class=HTMLResponse)
async def form(request: Request, brand: str | None = None) -> HTMLResponse:
    return templates.TemplateResponse(
        request,
        "copywriter.html",
        {
            **base_context(request, brand),
            "result": None,
            "error": None,
            "form_data": {},
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
                "product_name": imported.title,
                "category": "",
                "materials": "",
                "sizes": "",
                "colors": "",
                "price": imported.price,
                "key_benefits": imported.description,
                "target_keyword": "",
            }
            brand_profile = resolve_brand(brand or None)
            agent = CopywriterAgent(brand=brand_profile)
            result = await agent.write_product_page(ProductBrief(**form_data))
    except ImportError_ as exc:
        error = str(exc)
    except Exception as exc:
        error = format_error(exc)

    return templates.TemplateResponse(
        request,
        "copywriter.html",
        {
            **base_context(request, brand or None),
            "result": result,
            "error": error,
            "form_data": form_data,
            "imported": imported,
        },
    )


@router.post("", response_class=HTMLResponse)
async def submit(
    request: Request,
    brand: str = Form(default=""),
    product_name: str = Form(...),
    category: str = Form(default=""),
    materials: str = Form(default=""),
    sizes: str = Form(default=""),
    colors: str = Form(default=""),
    price: str = Form(default=""),
    key_benefits: str = Form(default=""),
    target_keyword: str = Form(default=""),
    imported_image_url: str = Form(default=""),
    imported_source_url: str = Form(default=""),
    imported_title: str = Form(default=""),
    imported_price: str = Form(default=""),
) -> HTMLResponse:
    form_data = {
        "product_name": product_name,
        "category": category,
        "materials": materials,
        "sizes": sizes,
        "colors": colors,
        "price": price,
        "key_benefits": key_benefits,
        "target_keyword": target_keyword,
    }
    result, error = None, None
    try:
        brand_profile = resolve_brand(brand or None)
        agent = CopywriterAgent(brand=brand_profile)
        brief = ProductBrief(**form_data)
        result = await agent.write_product_page(brief)
    except Exception as exc:
        error = format_error(exc)

    return templates.TemplateResponse(
        request,
        "copywriter.html",
        {
            **base_context(request, brand or None),
            "result": result,
            "error": error,
            "form_data": form_data,
            "imported": imported_from_form(
                image_url=imported_image_url,
                source_url=imported_source_url,
                title=imported_title,
                price=imported_price,
            ),
        },
    )
