# Deployer sur Hugging Face Spaces depuis ton iPhone

Tout se fait dans Safari, sans carte bancaire, sans CLI.
Resultat : une URL publique du type `https://huggingface.co/spaces/<ton-pseudo>/lelins-agency` que tu ouvres depuis n'importe ou.

## 1. Creer un compte Hugging Face (1 min)

1. Ouvre <https://huggingface.co/join>.
2. Cree un compte (email + mot de passe ou via Google/GitHub).
3. Confirme ton email.

## 2. Creer une cle API Mistral (2 min)

1. Ouvre <https://console.mistral.ai/>.
2. Cree un compte (gratuit, aucune carte demandee pour le free tier "Experiment").
3. Va dans **API Keys** -> **Create new key**.
4. **Copie la cle** (commence par `...`). On en a besoin a l'etape 4.

## 3. Creer le Space (2 min)

1. Sur Hugging Face, clique sur ta photo en haut a droite -> **New Space**.
2. Renseigne :
   - **Owner** : ton pseudo
   - **Space name** : `lelins-agency` (ou autre)
   - **License** : MIT (ou ce que tu veux)
   - **Select the SDK** : choisis **Docker** -> **Blank**
   - **Space hardware** : `CPU basic` (free)
   - **Visibility** : Public (ou Private si tu prefere)
3. Clique sur **Create Space**.

## 4. Connecter ton repo GitHub

Deux options. Le plus simple sur iPhone :

### Option A : Importer depuis GitHub (recommande)

1. Sur la page de ton Space fraichement cree, va dans l'onglet **Files**.
2. Clique sur **"+ Contribute"** -> **"Upload files"** ou utilise le bouton **"Import from GitHub"** s'il est propose.
3. Indique le repo `Julien-dao/Lelins` et la branche `claude/ecommerce-marketing-agency-JXaHa` (ou `main` apres merge de la PR).

### Option B : Copier le contenu

Si l'import GitHub n'est pas dispo, va dans l'onglet **Files** du Space et upload manuellement les fichiers du repo. HF Spaces accepte les uploads via le navigateur mobile.

Le Dockerfile et le frontmatter YAML dans `README.md` sont deja configures pour HF Spaces (port 7860, SDK Docker).

## 5. Ajouter ta cle Mistral en secret

1. Sur la page du Space, va dans **Settings** -> **Variables and secrets**.
2. Section **Secrets** -> **New secret** :
   - Name : `MISTRAL_API_KEY`
   - Value : colle la cle copiee a l'etape 2
3. **Save**.
4. Le Space va automatiquement se rebuild.

## 6. Patienter le build (~2-3 min)

L'onglet **Logs** te montre la construction de l'image Docker, puis le demarrage d'uvicorn. Quand tu vois `Application startup complete`, c'est pret.

## 7. Ouvrir l'app

L'URL publique est : `https://<ton-pseudo>-lelins-agency.hf.space`
(ou bien le bouton **App** depuis la page du Space ouvre directement l'iframe).

Tu peux la mettre en favori dans Safari ou l'ajouter a ton ecran d'accueil iOS (icone de partage -> "Sur l'ecran d'accueil") pour l'utiliser comme une app.

## Limites a connaitre

- **Free tier Mistral** : quotas RPM/jour. Si tu prends `429`, l'erreur s'affiche proprement dans l'UI. Attends quelques secondes / minutes.
- **Free tier HF Spaces** : 16 Go RAM partage, CPU. Le premier chargement peut prendre 20s si le Space etait en sommeil.
- **Visibilite** : si tu mets le Space en Public, n'importe qui pourra appeler ton agence (et donc consommer tes quotas Mistral). Pour un usage prive, mets-le en **Private** : seul toi pourras y acceder une fois connecte a HF.

## Mise a jour

Si tu modifies le code (depuis ton repo GitHub), HF peut etre re-synchronise via l'onglet **Files** -> **"Sync from GitHub"** ou en re-uploadant les fichiers. Le Space rebuild automatiquement.
