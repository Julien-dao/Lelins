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

## Astuces vitesse (Mac MPS)

Apple Silicon est ~3-5× plus lent que NVIDIA pour la diffusion. Pour itérer
plus vite, dans **Paramètres avancés** du formulaire :

- **Pas (steps)** : passer de 25 à **15-18** divise quasi par 2 le temps
  pour un visuel d'aperçu. Remonter à 30+ pour la version finale validée.
- **Résolution** : 768×768 au lieu de 1024×1024 pour explorer ; 1024×1024+
  pour la version finale.
- **Verrouillage facial** : à `0.5` au lieu de `0.7` laisse plus de variété
  visage ; à `0.0` désactivé tu retires l'IP-Adapter (un peu plus rapide).
- **Premier run lent** : 7 Go de SDXL + 2 Go d'IP-Adapter à télécharger
  une seule fois. Les générations suivantes sont 10-50× plus rapides.

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
