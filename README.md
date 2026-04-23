# Lelins Studio

Studio de création locale pour créateurs de contenus — édition image, aperçus réseaux sociaux, légendes enrichies.

Application 100 % navigateur, sans backend ni API externe. Les tokens OAuth éventuellement
utilisés pour la publication restent stockés localement (`localStorage`) et ne quittent jamais
la machine.

## Fonctionnalités (itération 1)

- Import d'images et recadrage selon le format cible (IG Feed/Story/Reel, TikTok, YT Shorts, X, LinkedIn, Facebook).
- Éditeur canvas basé sur Konva : calques texte et sous-titres, déplacement, redimensionnement, rotation, styles (police, taille, gras/italique, contour, fond pilule, opacité).
- Filtres image : 8 presets + sliders luminosité / contraste / saturation / teinte / flou, et toggles N&B / sépia / inverser.
- Éditeur de légende avec autocomplétion `#hashtag` et `@mention`, compteurs par réseau.
- Aperçus fidèles : Instagram, TikTok, YouTube Shorts, X, LinkedIn.
- Export PNG/JPEG à la résolution native du format.
- Partage natif via Web Share API (mobile) avec fallback téléchargement + copie de légende.
- Panneau comptes : ajout de tokens OAuth utilisateur, sélection multi-comptes pour publication (publishers stub, câblage en itération 2).

## Lancer en local

```bash
npm install
npm run dev        # http://localhost:5173
```

## Build de production

```bash
npm run build
npm run preview
```

Le dossier `dist/` peut être servi par n'importe quel serveur statique (Nginx, Caddy, `python -m http.server`).

## Roadmap itération 2

- Timeline vidéo / reel (FFmpeg.wasm) : découpe, sous-titres burn-in, export MP4/WebM.
- Implémentations concrètes des publishers câblés aux tokens utilisateur :
  - Instagram Graph API (Business/Creator)
  - TikTok Content Posting API
  - YouTube Data API v3
  - X API v2
  - LinkedIn Marketing API
- Programmation de publications (file d'attente locale).

## Architecture

```
src/
├── App.tsx                     Shell principal (toolbar + 2 sidebars + canvas)
├── components/
│   ├── Editor/                 Canvas Konva, calques, inspecteur texte, filtres
│   ├── Caption/                Éditeur de légende + compteurs
│   ├── Preview/                Aperçus par réseau
│   ├── Accounts/               Gestion des comptes + tokens
│   └── Layout/                 Sidebar générique
├── store/editorStore.ts        Store Zustand unique
├── lib/                        Formats, filtres, légende, publishers, export
├── hooks/                      Utilitaires React
└── types.ts                    Types partagés
```
