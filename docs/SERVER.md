# Serveur web — interface navigateur

Une mini-app FastAPI permet d'utiliser le générateur depuis ton navigateur
(desktop ou mobile), sans taper de commande pour chaque génération.

## Lancer le serveur (sur ton Mac)

Depuis la racine du dépôt, après avoir installé les dépendances
(`pip install -r scripts/requirements.txt`) :

```bash
python scripts/serve.py
```

Sortie attendue :

```
Serveur Lelins
  → http://127.0.0.1:8000
  (local uniquement ; pour accéder depuis le téléphone, relancer avec --host 0.0.0.0)

INFO:     Uvicorn running on http://127.0.0.1:8000 (Press CTRL+C to quit)
INFO:     Application startup complete.
```

Ouvrir **http://127.0.0.1:8000** dans Safari / Chrome / Firefox.

Pour arrêter : `CTRL+C` dans le terminal.

## Ce que tu peux faire dans l'interface

L'interface est divisée en trois onglets :

### 1. Visuel produit
- Choisir un mannequin (Character) dans la liste, ou laisser vide pour utiliser
  un preset générique.
- Décrire le vêtement (texte libre).
- Choisir le décor, la pose.
- Régler la force du verrouillage facial (slider).
- Cliquer **Générer** → l'image s'affiche.

### 2. Portrait mannequin
- Choisir un profil dans `characters/`.
- Cocher "tirer une nouvelle seed" pour explorer un nouveau visage.
- Le portrait est généré et **enregistré dans `characters/<nom>.png`**.
  La seed et le `portrait_path` sont écrits dans le profil JSON.

### 3. Essayage virtuel
- Uploader une image du mannequin (issue de l'onglet "Visuel produit").
- Uploader la photo de ton produit réel (packshot).
- Soit uploader un masque PNG, soit choisir une zone auto (`hip` ou `torso`).
- Décrire le vêtement.
- Cliquer **Lancer l'essayage**.

## Notes de performance

- Le serveur démarre en **~1 s**.
- Les modèles sont chargés **paresseusement** :
  - Première génération produit : ~30-90 s (chargement SDXL base).
  - Première génération try-on : ~60-120 s (chargement SDXL inpainting).
  - Premier verrouillage facial : ~20 s en plus (téléchargement IP-Adapter
    au premier run, puis mise en cache).
- Générations suivantes : ~10-90 s selon ton matériel
  (voir [`USAGE.md`](USAGE.md)).
- Les modèles **restent en mémoire** entre requêtes — ne pas relancer le
  serveur entre deux générations.

## Suivi en direct (streaming SSE)

L'interface utilise du **Server-Sent Events** : pendant chaque génération tu
vois en temps réel
- les messages de chargement (modèle, IP-Adapter…) ;
- une barre de progression qui avance pas à pas ;
- l'estimation du temps restant (ETA) qui se précise après les premiers pas.

Endpoints streaming exposés (`/api/generate-stream`,
`/api/generate-character-stream`, `/api/tryon-stream`) — payload texte/SSE
avec ces types d'événements :

| Type      | Contenu                                                       |
|-----------|---------------------------------------------------------------|
| `status`  | `{message}` — étape de chargement en cours                    |
| `ready`   | `{message, total}` — modèle prêt, génération démarre          |
| `step`    | `{step, total, percent, elapsed, eta}` — un pas de diffusion  |
| `done`    | `{image_url, seed, elapsed_seconds, ...}` — image prête       |
| `error`   | `{message}` — exception levée                                 |

Les anciens endpoints synchrones (`/api/generate`, etc.) restent disponibles
pour des appels API qui ne veulent pas parser le SSE — ils retournent du
JSON classique en fin de génération.

## ⚡ Mode rapide (SDXL Turbo)

Pour itérer rapidement — particulièrement utile sur **Mac M1 8 Go** ou tout
matériel modeste — l'interface propose un toggle **« ⚡ Mode rapide »**
dans les onglets Visuel produit et Portrait mannequin.

Quand activé :

- Bascule sur **SDXL Turbo** (modèle dérivé optimisé pour très peu de pas)
- Force `steps = 4` et `guidance_scale = 0` (paramètres optimaux Turbo)
- Génère **3-5× plus vite** qu'en mode standard
- Qualité légèrement inférieure (parfait pour l'exploration, moins pour le
  rendu final destiné au catalogue)

Indicatif sur Mac MPS, 768×768 :

| Matériel        | Mode standard (25 pas) | Mode rapide (4 pas) |
|-----------------|-----------------------|---------------------|
| M1 8 Go         | ~75 s                 | **~20 s**           |
| M2 16 Go        | ~45 s                 | **~12 s**           |
| M3 Max          | ~20 s                 | **~5 s**            |

### ⚠️ Bascule entre les modes (important sur 8 Go RAM)

Activer le mode rapide pour la première fois déclenche le **téléchargement
de SDXL Turbo** (~7 Go, identique à SDXL base). Une seule fois.

Sur **Mac 8 Go**, garder les deux modèles en RAM est impossible : à chaque
bascule entre standard et rapide, le serveur **décharge l'ancien modèle**
(rapide) puis recharge le nouveau (~30-60 s). C'est documenté par un message
clair dans la barre de progression. Reste sur un seul mode pour une session
de travail efficace.

Sur **Mac 16 Go+**, la bascule est aussi opérée mais on pourrait à terme
garder les deux en cache. Pour l'instant on décharge systématiquement
(comportement uniforme).

## Astuces vitesse (Mac MPS)

Apple Silicon est ~3-5× plus lent que NVIDIA pour la diffusion. Pour itérer
plus vite :

- **Mode rapide** (cf. ci-dessus) — le levier le plus efficace.
- **Pas (steps)** : en mode standard, passer de 25 à **15-18** divise quasi
  par 2 le temps pour un visuel d'aperçu. Remonter à 30+ pour la version
  finale validée.
- **Résolution** : 768×768 au lieu de 1024×1024 pour explorer ; 1024×1024+
  pour la version finale.
- **Verrouillage facial** : à `0.5` au lieu de `0.7` laisse plus de variété
  visage ; désactivé (case « Désactiver le verrouillage du visage ») tu
  retires l'IP-Adapter (un peu plus rapide, ~10 % de RAM en moins).
- **Premier run lent** : 7 Go de SDXL + 2 Go d'IP-Adapter à télécharger
  une seule fois. Les générations suivantes sont 10-50× plus rapides.

### Spécifique Mac M1 / M2 8 Go

- Lance **uniquement Safari + Terminal** (quitte Chrome, Slack, Photoshop,
  Spotify avant de lancer le serveur).
- **Préfère le mode rapide** par défaut, switch vers standard seulement pour
  le rendu final.
- Garde la résolution à **512×512 ou 768×768**, évite 1024×1024.
- N'utilise pas le try-on en parallèle d'une génération en cours (deux
  modèles SDXL en mémoire = swap garanti = lenteur extrême).

## Erreur "MPS backend out of memory"

Sur 8-16 Go RAM, certaines combinaisons (résolution élevée + IP-Adapter +
modèle inpainting) peuvent dépasser la limite mémoire MPS d'Apple.

**Le serveur est désormais protégé sur deux niveaux** :

1. **Désactivation du plafond MPS** — `scripts/serve.py` positionne
   automatiquement `PYTORCH_MPS_HIGH_WATERMARK_RATIO=0.0` au démarrage.
   Cela permet à PyTorch de demander plus de RAM que la limite par défaut
   (~70 % de la RAM totale) ; macOS gère le swap sur SSD si nécessaire.
   Lent mais évite le crash.

2. **Retry automatique à 512×512** — si une génération hit l'OOM malgré tout,
   le serveur libère la mémoire et **relance automatiquement à 512×512**.
   Tu vois le message dans la barre de progression :
   « Mémoire insuffisante à 1024×1024, on réessaie à 512×512… »

### Si l'OOM persiste à 512×512

C'est rare mais possible. Solutions, par ordre :

1. **Quitter toutes les autres apps** (Chrome surtout — vérifier dans le
   moniteur d'activité, trier par mémoire).
2. **Activer le mode rapide** (SDXL Turbo + 512×512 par défaut, le plus léger).
3. **Désactiver le verrouillage du visage** (case « Désactiver le verrouillage
   du visage » dans Paramètres avancés) — économise ~1-2 Go.
4. **Redémarrer le serveur** (`CTRL+C` puis relancer) — purge la RAM.
5. En dernier recours, **redémarrer le Mac** — libère le swap accumulé.

## Accéder depuis le téléphone

### Option A — Réseau local (Wi-Fi commun)

Sur le Mac :

```bash
python scripts/serve.py --host 0.0.0.0
```

Trouver l'IP locale du Mac (Réglages → Wi-Fi → ⓘ → Adresse IP, ex. `192.168.1.42`).

Sur le téléphone, ouvrir :

```
http://192.168.1.42:8000
```

⚠️ Le téléphone doit être sur le **même Wi-Fi** que le Mac.

### Option B — Tailscale (n'importe où, recommandé)

[Tailscale](https://tailscale.com) crée un VPN privé chiffré entre tes appareils.

1. Installer Tailscale sur le Mac et sur le téléphone (apps officielles).
2. Se connecter avec le même compte sur les deux.
3. Lancer le serveur : `python scripts/serve.py --host 0.0.0.0`
4. Récupérer l'IP Tailscale du Mac (visible dans l'app Tailscale, format
   `100.x.x.x`).
5. Sur le téléphone : `http://100.x.x.x:8000`

Avantages : marche partout (4G, Wi-Fi extérieur), chiffré, gratuit pour usage perso.

### Option C — Tunnel public (cas exceptionnel)

Pour partager temporairement à quelqu'un d'externe (designer, photographe…),
utiliser **Cloudflare Tunnel** :

```bash
# Installer cloudflared (brew install cloudflare/cloudflare/cloudflared)
cloudflared tunnel --url http://localhost:8000
```

Cloudflare donne une URL publique HTTPS aléatoire, valide tant que la commande
tourne. ⚠️ Tout le monde avec l'URL peut générer — à n'utiliser que ponctuellement.

## Personnaliser le port

```bash
python scripts/serve.py --port 8080
```

## Mode développement (auto-reload)

Si tu modifies les fichiers du serveur (`server/app.py`, `server/templates/...`)
et veux voir les changements sans relancer manuellement :

```bash
python scripts/serve.py --reload
```

À ne pas utiliser en usage normal — l'auto-reload coûte un peu de RAM.

## API REST (pour automatisation)

Le serveur expose aussi des endpoints JSON, utilisables sans l'UI :

| Endpoint                       | Méthode | Description                                |
|--------------------------------|---------|--------------------------------------------|
| `/api/status`                  | GET     | État du serveur, matériel, modèles chargés |
| `/api/generate`                | POST    | Génère un visuel produit (form-data)       |
| `/api/generate-character`      | POST    | Génère un portrait                         |
| `/api/tryon`                   | POST    | Essayage virtuel (multipart/form-data)     |
| `/outputs/<file>`              | GET     | Sert les images générées                   |

Documentation interactive auto-générée par FastAPI :
**http://127.0.0.1:8000/docs**

Exemple curl :

```bash
curl -X POST http://127.0.0.1:8000/api/generate \
  -F "garment=black cotton boxer briefs" \
  -F "character_path=characters/lelins-main.json" \
  -F "background=studio_white"
```

Réponse JSON :

```json
{
  "image_url": "/outputs/gen-20260427-143012-7821.png",
  "seed": 1287634,
  "elapsed_seconds": 24.3,
  "prompt": "...",
  "face_lock": true
}
```

## Dépannage

### `ERREUR : uvicorn n'est pas installé`

```bash
pip install -r scripts/requirements.txt
```

### Port 8000 déjà utilisé

```bash
python scripts/serve.py --port 8080
```

### "Connection refused" depuis le téléphone

Vérifier :
1. Le serveur est lancé avec `--host 0.0.0.0` (et non `127.0.0.1`).
2. Téléphone et Mac sont sur le même Wi-Fi (ou même réseau Tailscale).
3. Le pare-feu macOS n'a pas bloqué Python : Réglages → Confidentialité et
   sécurité → Coupe-feu → autoriser Python.

### Première génération bloquée à "loading…"

Normal au premier lancement : le téléchargement de SDXL (~7 Go) peut prendre
plusieurs minutes. Regarder le terminal — uvicorn affiche la progression du
téléchargement HuggingFace.

### Le serveur consomme beaucoup de RAM

Normal aussi : SDXL en mémoire = ~6-8 Go. Si tu utilises aussi le try-on,
SDXL inpainting ajoute ~6 Go. Sur Mac 16 Go, ça tient mais c'est tendu —
fermer les autres apps gourmandes (navigateur avec beaucoup d'onglets, etc.).
