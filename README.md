# Lelins Studio

Studio de création locale pour créateurs de contenus — édition **image et vidéo**, aperçus
réseaux sociaux, légendes enrichies, publication directe avec vos propres tokens OAuth.

Application 100 % navigateur, sans backend. Les binaires FFmpeg.wasm sont servis depuis
`/public/ffmpeg/` (copiés en local au `postinstall`) et les tokens OAuth restent dans
`localStorage` de votre navigateur — rien n'est envoyé ailleurs que vers les API officielles des
réseaux que vous avez configurés.

## Fonctionnalités

### Itération 1 — image
- Import d'images et recadrage selon le format cible (IG Feed/Story/Reel, TikTok, YT Shorts, X, LinkedIn, Facebook).
- Éditeur canvas (Konva) : calques texte et sous-titres, déplacement, redimensionnement, rotation, styles complets (police, taille, gras/italique, contour, fond pilule, opacité).
- Filtres : 8 presets + sliders luminosité / contraste / saturation / teinte / flou, toggles N&B / sépia / inverser.
- Éditeur de légende avec autocomplétion `#hashtag` / `@mention` et compteurs par réseau.
- Aperçus fidèles : Instagram, TikTok, YouTube Shorts, X, LinkedIn.
- Export PNG/JPEG à la résolution native du format, Web Share API mobile.

### Itération 2 — vidéo et publication
- Import vidéo (MP4/WebM/MOV) avec timeline de découpe (scrubber + poignées trim start/end).
- Prévisualisation de la vidéo dans le canvas avec calques texte/sous-titres et filtres appliqués en live.
- Export MP4 via FFmpeg.wasm (single-thread, entièrement local) :
  - découpe selon la plage choisie
  - overlay texte/sous-titres burn-in (composite d'un PNG alpha)
  - filtres appliqués via `eq`, `hue`, `boxblur`, `colorchannelmixer`, etc.
  - scale + crop pour matcher exactement les dimensions du format cible
  - encodage `libx264 + aac`, `faststart`
- Publishers câblés aux tokens OAuth utilisateur :
  - **YouTube Data API v3** — upload resumable, brouillon privé créé
  - **Facebook Graph API** — photos/videos directement via multipart
  - **Instagram Graph API** — création + publication (exige une URL publique pour le média)
  - **TikTok Content Posting API** — inbox upload (la publication finale se fait dans l'app TikTok)
  - **X** et **LinkedIn** — identifiés CORS-blocked avec message clair (proxy local requis)

### Itération 3 — sous-titres auto + proxy local
- **Sous-titres automatiques** par Whisper (transformers.js, modèles tiny/base/small) :
  - extraction audio mono 16 kHz via FFmpeg.wasm
  - transcription avec timestamps, multilingue (auto / fr / en / es / de / it / pt)
  - éditeur de segments (texte modifiable, suppression, jump-to-time)
  - rendu live des sous-titres dans le canvas pendant la lecture
  - burn-in à l'export via fichier ASS et filter `subtitles=`
  - style configurable : taille, position (haut/milieu/bas), couleurs, fond (aucun/bloc/pilule)
  - Whisper en chargement dynamique : ne pèse que sur le bundle quand l'utilisateur clique « Générer »
- **Proxy local** (`proxy/server.mjs`) pour débloquer X et LinkedIn :
  - Node script de ~120 lignes, écoute sur `127.0.0.1:8088`
  - relais transparent vers `api.twitter.com`, `upload.twitter.com`, `api.linkedin.com`
  - CORS limité aux origines locales (`localhost:5173/4173`)
  - aucune persistance, aucun log de token
  - l'app détecte automatiquement le proxy via `/_health` et bascule X/LinkedIn dessus
  - implémentations complètes : X (INIT/APPEND/FINALIZE + tweet) et LinkedIn (UGC posts API)

## Lancer en local

```bash
npm install          # copie aussi FFmpeg.wasm (~32 Mo) dans public/ffmpeg/
npm run dev          # http://localhost:5173
```

Pour publier sur X ou LinkedIn, lancez le proxy local en parallèle :

```bash
node proxy/server.mjs   # http://127.0.0.1:8088
```

## Build de production

```bash
npm run build
npm run preview
```

Le dossier `dist/` peut être servi par n'importe quel serveur statique (Nginx, Caddy,
`python -m http.server`). **Important** : servir avec les headers suivants pour que FFmpeg.wasm
fonctionne proprement (la variante single-thread actuelle n'en a pas besoin, mais c'est plus
sûr pour l'avenir multi-thread) :

```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: credentialless
```

## Publier vers les réseaux

L'app n'a aucun backend. Pour chaque réseau que vous voulez utiliser, vous devez :

1. Créer votre propre **app développeur** chez le fournisseur (Google Cloud, Meta, TikTok, etc.).
2. Générer un **access token** avec les scopes nécessaires.
3. Aller dans l'onglet **Comptes** de l'app et coller le token (+ ID du compte/Page pour IG/FB).
4. Cocher les comptes cibles, puis cliquer **Publier**.

| Réseau      | Prérequis                                  | Statut depuis navigateur             |
|-------------|---------------------------------------------|--------------------------------------|
| YouTube     | OAuth scope `youtube.upload`                | ✅ direct                             |
| Facebook    | Page ID + Page Access Token                 | ✅ direct                             |
| Instagram   | IG Business Account ID + URL publique média | ✅ direct (hébergement média requis)  |
| TikTok      | App approuvée, scope `video.upload`         | ✅ inbox (publication finale in-app)  |
| X (Twitter) | X API v2 tier payant + proxy local          | ✅ via proxy local                    |
| LinkedIn    | Marketing API + URN auteur + proxy local    | ✅ via proxy local                    |

## Roadmap

- File d'attente de publications programmées (cron local).
- Presets additionnels (LUTs, overlays stickers, watermark).
- Aperçus video pour les réseaux verticaux (lecture inline).
- Tests E2E (Playwright) sur le pipeline image + vidéo.

## Architecture

```
src/
├── App.tsx                     Shell principal (toolbar + 2 sidebars + canvas + timeline)
├── components/
│   ├── Editor/                 Canvas Konva, calques, inspecteur texte, filtres, timeline vidéo
│   ├── Caption/                Éditeur de légende + compteurs
│   ├── Preview/                Aperçus par réseau
│   ├── Accounts/               Gestion des comptes + tokens
│   └── Layout/                 Sidebar générique
├── store/editorStore.ts        Store Zustand unique (media image|vidéo, layers, filter, caption, accounts)
├── lib/
│   ├── ffmpeg.ts               Singleton FFmpeg.wasm (lazy)
│   ├── videoExport.ts          Pipeline de rendu vidéo (trim + filtres + overlay + sous-titres)
│   ├── overlayRender.ts        Rend les calques texte en PNG transparent pour burn-in
│   ├── audio.ts                Extraction audio 16 kHz pour Whisper
│   ├── whisper.ts              Loader transformers.js (lazy import)
│   ├── subtitles.ts            Génération ASS pour FFmpeg
│   ├── publishers.ts           Implémentations YouTube, IG, FB, TikTok, X, LinkedIn + proxy detect
│   ├── formats.ts              Specs réseaux
│   ├── filters.ts              Presets et helpers CSS
│   ├── caption.ts              Parse #/@, autocomplétion
│   └── export.ts               Téléchargement + Web Share API
├── hooks/                      Utilitaires React
└── types.ts                    Types partagés

proxy/
└── server.mjs                  Proxy Node 100 % standalone pour X et LinkedIn
```
