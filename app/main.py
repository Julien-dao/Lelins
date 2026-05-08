from pathlib import Path

from fastapi import FastAPI, Request
from fastapi.responses import HTMLResponse
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates

from app.brands import BRANDS
from app.config import get_settings
from app.routers import ads, copywriter, email, social

BASE_DIR = Path(__file__).resolve().parent

app = FastAPI(title="Agence virtuelle Lelins", version="0.1.0")
app.mount(
    "/static",
    StaticFiles(directory=str(BASE_DIR.parent / "static")),
    name="static",
)
templates = Jinja2Templates(directory=str(BASE_DIR / "templates"))

app.include_router(copywriter.router)
app.include_router(email.router)
app.include_router(social.router)
app.include_router(ads.router)


@app.get("/", response_class=HTMLResponse)
async def home(request: Request) -> HTMLResponse:
    settings = get_settings()
    return templates.TemplateResponse(
        request,
        "index.html",
        {
            "brands": list(BRANDS.values()),
            "active_brand": settings.default_brand,
            "mistral_configured": bool(settings.mistral_api_key),
            "model": settings.mistral_model,
        },
    )


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok"}
