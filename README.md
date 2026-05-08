# Lelins &mdash; Agence virtuelle de marketing direct

Vente en ligne de sous-vetements et vetements homme. Ce repo heberge une
**agence marketing virtuelle** : une petite app FastAPI ou plusieurs "agents"
IA (copywriter, email, social media, publicite) generent du contenu pour des
boutiques e-commerce de produits physiques.

L'app fonctionne sur le **free tier de Mistral** (`mistral-small-latest`).
Architecture pensee pour Lelins en premier, mais facilement generalisable a
d'autres marques.

## Agents disponibles

| Agent | Page | Sortie |
| --- | --- | --- |
| Copywriter fiches produits | `/copywriter` | Titre SEO, meta, description longue, FAQ |
| Email marketing | `/email` | Welcome, panier abandonne, post-achat, win-back, drops |
| Social media | `/social` | Posts Instagram / TikTok / Facebook / LinkedIn |
| Publicites | `/ads` | Creatives Meta Ads, Google Search/PMax, TikTok Ads |

Chaque agent reutilise le profil de marque (`app/brands/`) pour calibrer son
ton, son audience, ses do/dont.

## Demarrage rapide

### 1. Cle API Mistral (gratuite)

1. Cree un compte sur <https://console.mistral.ai/>.
2. Genere une cle API.
3. Copie `.env.example` vers `.env` et renseigne `MISTRAL_API_KEY`.

```bash
cp .env.example .env
# puis edite .env
```

### 2. Installer les dependances

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

### 3. Lancer l'app

```bash
uvicorn app.main:app --reload
```

Ouvre <http://127.0.0.1:8000>. La doc OpenAPI est sur `/docs`.

## Structure

```
app/
  main.py             # FastAPI + montage UI / routers
  config.py           # settings (.env)
  llm/
    mistral_client.py # client HTTP Mistral
  brands/
    base.py           # BrandProfile (pydantic)
    lelins.py         # profil Lelins
  agents/
    base.py           # BaseAgent (system prompt + run)
    copywriter.py
    email.py
    social.py
    ads.py
  routers/
    copywriter.py     # GET form + POST submit
    email.py
    social.py
    ads.py
  templates/          # Jinja2
static/style.css
```

## Ajouter une marque

1. Creer `app/brands/<slug>.py` :

```python
from app.brands.base import BrandProfile

MA_MARQUE = BrandProfile(
    slug="ma_marque",
    name="Ma Marque",
    tagline="...",
    mission="...",
    audience="...",
    tone="...",
    do=[...],
    dont=[...],
)
```

2. L'enregistrer dans `app/brands/__init__.py` :

```python
from app.brands.ma_marque import MA_MARQUE
BRANDS["ma_marque"] = MA_MARQUE
```

La marque apparait automatiquement dans le selecteur de chaque page.

## Limites a garder en tete

- **Free tier Mistral** : quotas RPM/jour, attends-toi a des `429` si tu
  spam les requetes. L'erreur est rendue proprement dans l'UI.
- **Pas de stockage** : aucun contenu genere n'est sauvegarde, copie-colle
  ce qui te plait. Ajouter une persistance (SQLite/Postgres) est trivial si
  besoin.
- **Pas de garantie de conformite** : pour les pubs sante / finance / etc.,
  fais relire les sorties par un humain avant publication.

## Roadmap possible

- Export Markdown / CSV des resultats
- Mode batch (CSV de produits -> N fiches)
- Connecteur Shopify / WooCommerce (lecture catalogue)
- Calendrier editorial social media
- Multi-LLM (Groq, Gemini) avec fallback automatique
