from fastapi import APIRouter, File, Form, Request, UploadFile
from fastapi.responses import HTMLResponse

from app.image_studio import (
    ImageStudioError,
    generate_mannequin,
    studio_configured,
    to_data_uri,
    try_on,
)
from app.routers._helpers import base_context, templates

router = APIRouter(prefix="/studio", tags=["studio"])

MAX_UPLOAD_BYTES = 8 * 1024 * 1024


def _context(request: Request, **extra) -> dict:
    return {
        **base_context(request),
        "studio_configured": studio_configured(),
        "mannequin_url": None,
        "result_url": None,
        "garment_preview": None,
        "error": None,
        "prompt": "",
        "category": "bottoms",
        **extra,
    }


@router.get("", response_class=HTMLResponse)
async def form(request: Request) -> HTMLResponse:
    return templates.TemplateResponse(request, "studio.html", _context(request))


@router.post("", response_class=HTMLResponse)
async def generate(
    request: Request,
    prompt: str = Form(...),
    category: str = Form(default="bottoms"),
    garment: UploadFile = File(...),
) -> HTMLResponse:
    garment_preview = None
    mannequin_url = None
    result_url = None
    error = None

    try:
        content = await garment.read()
        if not content:
            raise ImageStudioError("Aucun fichier produit recu.")
        if len(content) > MAX_UPLOAD_BYTES:
            raise ImageStudioError("Image trop lourde (max 8 Mo).")
        content_type = garment.content_type or "image/png"
        if not content_type.startswith("image/"):
            raise ImageStudioError("Le fichier produit doit etre une image.")

        garment_data_uri = to_data_uri(content, content_type)
        garment_preview = garment_data_uri

        mannequin_url = await generate_mannequin(prompt)
        result_url = await try_on(mannequin_url, garment_data_uri, category)
    except ImageStudioError as exc:
        error = str(exc)
    except Exception as exc:  # noqa: BLE001
        error = f"Erreur inattendue : {exc}"

    return templates.TemplateResponse(
        request,
        "studio.html",
        _context(
            request,
            prompt=prompt,
            category=category,
            garment_preview=garment_preview,
            mannequin_url=mannequin_url,
            result_url=result_url,
            error=error,
        ),
    )
