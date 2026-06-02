import html as _html
import json
import re
from dataclasses import dataclass
from urllib.parse import urljoin, urlparse

import httpx

_META_RE = re.compile(r"<meta\b[^>]*>", re.IGNORECASE)
_ATTR_RE = re.compile(r'([a-zA-Z:_-]+)\s*=\s*"([^"]*)"')
_JSONLD_RE = re.compile(
    r'<script\b[^>]*type\s*=\s*["\']application/ld\+json["\'][^>]*>(.*?)</script>',
    re.IGNORECASE | re.DOTALL,
)
_TITLE_RE = re.compile(r"<title[^>]*>(.*?)</title>", re.IGNORECASE | re.DOTALL)
_WS_RE = re.compile(r"\s+")


class ImportError_(Exception):
    pass


@dataclass
class ProductImport:
    title: str = ""
    description: str = ""
    price: str = ""
    image_url: str = ""
    source_url: str = ""

    @property
    def is_empty(self) -> bool:
        return not (self.title or self.description or self.price)

    @property
    def has_preview(self) -> bool:
        return bool(self.image_url or self.title or self.price)


def imported_from_form(
    image_url: str = "",
    source_url: str = "",
    title: str = "",
    price: str = "",
) -> ProductImport | None:
    """Reconstruit un ProductImport a partir des champs caches d'un formulaire."""
    obj = ProductImport(
        title=title or "",
        price=price or "",
        image_url=image_url or "",
        source_url=source_url or "",
    )
    return obj if obj.has_preview else None


def _clean(value: str) -> str:
    if not value:
        return ""
    return _WS_RE.sub(" ", _html.unescape(value)).strip()


def _parse_meta(html_text: str) -> dict:
    metas: dict[str, str] = {}
    for tag in _META_RE.findall(html_text):
        attrs = dict(_ATTR_RE.findall(tag))
        key = attrs.get("property") or attrs.get("name")
        content = attrs.get("content")
        if key and content is not None:
            metas[key.lower()] = content
    return metas


def _iter_products(data):
    if isinstance(data, list):
        for item in data:
            yield from _iter_products(item)
    elif isinstance(data, dict):
        if "@graph" in data:
            yield from _iter_products(data["@graph"])
        raw_type = data.get("@type")
        types = raw_type if isinstance(raw_type, list) else [raw_type]
        if any(str(t).lower() == "product" for t in types if t):
            yield data


def _parse_jsonld(html_text: str) -> dict:
    for block in _JSONLD_RE.findall(html_text):
        try:
            data = json.loads(block.strip())
        except (json.JSONDecodeError, ValueError):
            continue
        for product in _iter_products(data):
            return product
    return {}


def _extract_image(value) -> str:
    if isinstance(value, str):
        return value
    if isinstance(value, list) and value:
        return _extract_image(value[0])
    if isinstance(value, dict):
        return value.get("url", "") or value.get("contentUrl", "")
    return ""


def _extract_price(offers) -> str:
    if isinstance(offers, list):
        for offer in offers:
            price = _extract_price(offer)
            if price:
                return price
        return ""
    if isinstance(offers, dict):
        price = offers.get("price") or offers.get("lowPrice") or ""
        currency = offers.get("priceCurrency", "")
        if price:
            return f"{price} {currency}".strip()
    return ""


def _og_price(meta: dict) -> str:
    amount = meta.get("product:price:amount") or meta.get("og:price:amount")
    if not amount:
        return ""
    currency = meta.get("product:price:currency") or meta.get("og:price:currency") or ""
    return f"{amount} {currency}".strip()


def _normalize_url(url: str) -> str:
    url = url.strip()
    if not url:
        raise ImportError_("Aucune URL fournie.")
    if not re.match(r"^https?://", url, re.IGNORECASE):
        url = "https://" + url
    parsed = urlparse(url)
    if not parsed.netloc:
        raise ImportError_("URL invalide.")
    return url


def parse_product(html_text: str, source_url: str) -> ProductImport:
    meta = _parse_meta(html_text)
    product = _parse_jsonld(html_text)

    title = product.get("name") or meta.get("og:title") or ""
    if not title:
        match = _TITLE_RE.search(html_text)
        title = match.group(1) if match else ""

    description = (
        product.get("description")
        or meta.get("og:description")
        or meta.get("description")
        or ""
    )
    image = _extract_image(product.get("image")) or meta.get("og:image") or ""
    if image:
        image = urljoin(source_url, image.strip())
    price = _extract_price(product.get("offers")) or _og_price(meta)

    return ProductImport(
        title=_clean(title),
        description=_clean(description),
        price=_clean(price),
        image_url=image,
        source_url=source_url,
    )


async def import_product(url: str) -> ProductImport:
    source_url = _normalize_url(url)
    headers = {
        "User-Agent": (
            "Mozilla/5.0 (compatible; LelinsAgency/1.0; +https://huggingface.co)"
        ),
        "Accept": "text/html,application/xhtml+xml",
    }
    try:
        async with httpx.AsyncClient(
            follow_redirects=True, timeout=15.0, headers=headers
        ) as client:
            response = await client.get(source_url)
            response.raise_for_status()
    except httpx.HTTPStatusError as exc:
        raise ImportError_(
            f"La page a répondu {exc.response.status_code}. Vérifie l'URL ou réessaie."
        ) from exc
    except httpx.HTTPError as exc:
        raise ImportError_(f"Impossible de charger la page : {exc}") from exc

    return parse_product(response.text, source_url)
