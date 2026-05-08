# Projet ANDREA — Formateur virtuel local pour Titre Pro FPA

> Brief de référence du projet — relu à chaque session.
> Auteur : Julien PERROT (directeur d'OF Qualiopi, La Réunion).
> Statut : brief initial, valide jusqu'à révision explicite.

---

## Identité du produit

**Nom : ANDREA**

Acronyme/évocation : référence directe à l'**andragogie** (la science de la
formation des adultes selon Knowles), prénom mixte permettant à chaque
utilisateur de s'identifier ou de personnaliser, mémorable, professionnel.

**Positionnement** : ANDREA n'est pas un assistant pédagogique. C'est un
**formateur expert en andragogie** qui maîtrise le référentiel du Titre
Professionnel Formateur Professionnel d'Adultes au bout des doigts, et qui
accompagne, forme, évalue et guide ses apprenants jusqu'à la certification.

## Contexte

Je suis Julien PERROT, directeur d'un organisme de formation (CFA + FPC,
certifié Qualiopi) à La Réunion (974). J'ai 15 ans d'expérience en formation
professionnelle d'adultes et je maîtrise le référentiel du Titre Professionnel
Formateur Professionnel d'Adultes (FPA).

ANDREA sera vendu sur Gumroad selon une stratégie en plusieurs tiers (voir
section Stratégie commerciale). Le produit doit donner immédiatement la
sensation d'un outil professionnel, soigné, premium. Pas un POC, pas une
démo technique : un véritable formateur virtuel abouti.

## Stratégie commerciale et tiers de licence

ANDREA est commercialisé en trois tiers, mais constitue **une seule et même
application** dont les fonctionnalités sont déverrouillées par clé de licence :

### ANDREA Découverte — 99 € (sortie cible : MVP)
- Onboarding personnalisé complet
- Choix avatar et voix
- Espace "Mes cours" : progression par compétence du référentiel FPA
- Conversation vocale et texte avec ANDREA
- Suivi de progression de base
- Sauvegarde locale et export des données

### ANDREA Pro — 199 € (sortie cible : v2)
- Tout le contenu Découverte
- Module évaluation des documents apprenant (DP, supports, productions)
- Simulateur de jury (3 épreuves)
- Module mises en situation professionnelle
- Coffre-fort documents structuré
- Tableau de bord progression avancé

### ANDREA Maître — 299 € (sortie cible : v3)
- Tout le contenu Pro
- Suivi de cohorte (pour formateurs FPA accompagnant leurs apprenants)
- Export des évaluations en PDF pour archivage Qualiopi
- Bibliothèque de scénarios pédagogiques avancés

> **Mise à jour 6/5/2026** : module VAE retiré du périmètre (décision Julien).
> Voir `docs/08-decisions.md`.

### Politique d'upgrade (CRITIQUE pour la confiance utilisateur)

- Un acheteur Découverte qui upgrade vers Pro paie la différence (99 € → 199 €,
  upgrade à 100 €)
- Un acheteur Pro qui upgrade vers Maître paie la différence (199 € → 299 €,
  upgrade à 100 €)
- Bundle "Le Titre en main" : Découverte + Pro + Maître à 449 € (au lieu de
  597 €)
- **Aucune perte de données entre les tiers** : la même installation passe
  d'un tier à l'autre par simple saisie d'une nouvelle clé

## Architecture pérenne et évolutive (NON NÉGOCIABLE)

ANDREA est conçu dès le départ pour évoluer en plusieurs versions sans
rupture pour l'utilisateur. L'architecture doit donc respecter ces
principes fondateurs **dès la v1 Découverte**, même si certaines
fonctionnalités ne sont activées qu'en v2/v3 :

### 1. Séparation stricte code / données

Les données utilisateur sont stockées dans le dossier système standard,
**jamais** dans le dossier d'installation :
- macOS : `~/Library/Application Support/ANDREA/`
- Windows : `%APPDATA%\ANDREA\`
- Linux : `~/.local/share/ANDREA/` (si Linux supporté)

Une désinstallation de l'application ne touche jamais ces données. Une
réinstallation les retrouve automatiquement.

### 2. Système de licence par tier (UNE seule application)

Il n'existe qu'**une seule application ANDREA**. Tout le code de tous les
tiers est embarqué dans la même appli. Une **clé de licence** déverrouille
les fonctionnalités correspondantes.

Format de clé proposé : `ANDREA-[TIER]-XXXX-XXXX-XXXX-XXXX`
- `ANDREA-DECO-...` : tier Découverte
- `ANDREA-PRO-...` : tier Pro
- `ANDREA-MAITRE-...` : tier Maître

La clé est :
- Saisie au premier lancement (avec lien "Acheter une clé" → Gumroad)
- Modifiable à tout moment dans les paramètres
- Validée hors-ligne par algorithme de signature cryptographique (pas de
  serveur central, l'application reste 100% locale)
- Liée à un email pour anti-piratage léger (saisi à l'installation,
  comparé au format de la clé)

À toi de me proposer un système de génération/validation de clé robuste
mais réalisable (suggestion : signature Ed25519 ou similaire, clé publique
embarquée dans l'app, génération via mon outil maison côté serveur Gumroad).

Quand un utilisateur saisit une nouvelle clé (par exemple Pro après avoir eu
Découverte), les nouveaux modules se déverrouillent immédiatement, sans
téléchargement, sans perte de données.

### 3. Versionnement de la base de données et migrations automatiques

La base SQLite locale contient une table `_metadata` indiquant sa version
de schéma. À chaque lancement, ANDREA :
1. Vérifie la version actuelle du schéma
2. Compare avec la version requise par l'appli installée
3. Si nécessaire, applique les migrations en séquence avec une barre de
   progression utilisateur ("Mise à niveau de votre profil… ✓")
4. Met à jour la version dans `_metadata`

Système de migrations idempotentes (style Alembic, Knex, ou simple maison)
permettant l'évolution future de la structure de données sans casser
l'existant.

### 4. Auto-updater intégré dès la v1

Configuration de l'updater officiel Tauri pour permettre les mises à jour
transparentes :
- Vérification au démarrage (configurable par l'utilisateur)
- Notification non intrusive
- Mise à jour en un clic
- Distribution via GitHub Releases (gratuit, fiable) ou serveur dédié

### 5. Export et import des données utilisateur

À tout moment, l'utilisateur peut :
- **Exporter** l'intégralité de ses données (profil + progression +
  documents + historique) dans un fichier `.andrea-backup` chiffré
- **Importer/restaurer** un backup sur la même machine ou une autre
  (changement de machine, réinstallation propre)

Cette fonctionnalité est essentielle pour la confiance utilisateur :
"mes données ne sont pas prisonnières".

### 6. Chiffrement local des documents sensibles

Les documents personnels de l'apprenant (Dossier Professionnel en cours de
rédaction notamment) sont chiffrés sur disque avec une clé dérivée du
profil utilisateur. Pas de mot de passe à mémoriser, mais protection en cas
de partage involontaire de la machine.

## Les trois rôles d'ANDREA

ANDREA n'est pas qu'un explicateur de cours. Il assume **trois rôles
indissociables** :

### 1. FORMATEUR (présent dès Découverte)

- Explique les notions du référentiel FPA en utilisant les principes de
  l'andragogie (Knowles : besoin de savoir, expérience comme ressource,
  orientation vers la résolution de problèmes, motivation intrinsèque)
- Adapte son discours au profil et à l'expérience de l'apprenant
- Utilise des techniques pédagogiques actives :
  - Maïeutique (questionnement socratique)
  - Apprentissage par problèmes
  - Études de cas réels du métier de formateur
  - Mises en situation professionnelle
  - Reformulation et vérification de compréhension
  - Synthèses et bilans réguliers
- Module sa progression selon les acquis observés

### 2. ÉVALUATEUR (déverrouillé en Pro)

- Évalue les acquis de l'apprenant en continu (évaluation formative)
- Propose des évaluations sommatives par compétence
- **Évalue les supports produits par l'apprenant** pour son passage du titre :
  - Dossier Professionnel (DP)
  - Présentations orales et supports
  - Études de cas et mises en situation
  - Tout document que l'apprenant souhaite faire relire
- Donne des retours structurés selon les critères réels du jury
- Identifie les écarts entre la production de l'apprenant et les attendus
  du référentiel
- Propose des axes d'amélioration concrets

### 3. GUIDE (déverrouillé en Pro, enrichi en Maître)

- Accompagne l'apprenant dans la **constitution de son Dossier Professionnel**
  étape par étape
- Connaît parfaitement les attendus du jury : ce qui est demandé, comment,
  dans quel format, avec quels critères
- Aide à structurer les preuves de compétences
- Prépare aux 3 épreuves de certification :
  - **Mise en Situation Professionnelle (MSP)** : conception, animation,
    évaluation d'une séquence de formation
  - **Entretien Technique** sur le DP
  - **Entretien Final** avec le jury (questionnement professionnel)
- Simule les conditions d'épreuve à la demande
- Suit la progression vers la certification dans un tableau de bord

## Maîtrise du référentiel (NON NÉGOCIABLE)

ANDREA doit **connaître par cœur** :

- Le **Référentiel d'Emploi, Activités et Compétences (REAC)** du titre FPA
- Le **Référentiel d'Évaluation (RE)** correspondant
- Les **2 CCP** :
  - CCP1 — Préparer et animer des actions de formation
  - CCP2 — Construire des parcours individualisés et accompagner les
    apprenants
- Les modalités d'évaluation détaillées
- Les critères du jury
- Les attendus du Dossier Professionnel
- Les évolutions réglementaires récentes du titre

À récupérer impérativement sur :
- France Compétences (rncp.francecompetences.fr) — fiche RNCP du titre FPA
- Site du ministère du Travail
- AFPA (qui gère historiquement ce titre)

Cette base de connaissance est la **colonne vertébrale du produit**. Sans
maîtrise référentielle parfaite, ANDREA n'a aucune valeur.

## Expérience utilisateur attendue

### Onboarding personnalisé (premier lancement)

L'utilisateur doit se sentir reconnu et accueilli :
- Saisie de la clé de licence (avec lien "Acheter une licence" → Gumroad)
- Demande son prénom et comment il souhaite être appelé
- Son contexte (apprenant FPA en CFA, candidat VAE, formateur en
  reconversion, professionnel cherchant à structurer sa pratique, etc.)
- Son niveau de départ (débutant total, expérience pratique sans formation,
  formateur en exercice, etc.)
- Son objectif (titre complet, CCP1 seul, CCP2 seul, simple montée en
  compétences sans certification)
- Sa cadence souhaitée (intensif 10h/semaine, régulier 3h/semaine, à mon
  rythme)
- Sa date d'épreuve si elle est connue (pour planification à rebours)
- Sauvegarde ce profil localement et l'utilise dans toutes les interactions
  futures

### Choix du formateur virtuel

L'utilisateur peut personnaliser ANDREA :
- **Nom** : conserver "ANDREA" ou personnaliser (le nom devient celui du
  formateur affiché)
- **Avatar** : galerie de photos pré-fournies (homme/femme, diversité
  d'âges et d'origines, **incluant représentation ultramarine** pour
  identification des apprenants réunionnais) + possibilité d'uploader sa
  propre image
- **Voix** : voix homme ou femme, française, naturelle. Plusieurs voix
  proposées avec écoute préalable d'un échantillon.
- **Tutoiement ou vouvoiement** : selon préférence
- Tous ces choix modifiables à tout moment dans les paramètres

### Pendant les sessions

- L'avatar du formateur est visible et "vivant" (animation discrète de
  respiration, indication visuelle quand il "parle" ou "écoute")
- Indicateur clair de l'état : "Je vous écoute…", "Je réfléchis…",
  "Je vous explique…"
- Possibilité d'interrompre ANDREA à tout moment (barge-in)
- Possibilité de basculer entre vocal et texte selon le contexte
- Historique des sessions consultable et reprenable
- Tableau de bord de progression par compétence du référentiel

### Espaces dédiés dans l'application

ANDREA propose plusieurs espaces clairement identifiés (certains verrouillés
selon le tier) :

**Tier Découverte :**
1. **Mes cours** : progression dans le référentiel par compétence
2. **Ma progression** : où j'en suis, ce qui reste, vue d'ensemble
3. **Mes paramètres** : avatar, voix, profil, licence
4. **Mon coffre-fort** (basique) : sauvegarde de mes notes

**Tier Pro (en plus) :**
5. **Mon Dossier Professionnel** : assistant à la constitution du DP,
   relecture, évaluation
6. **Mes mises en situation** : entraînement à la MSP avec scénarios
7. **Simulateur de jury** : simulations d'entretiens avec retours
8. **Mon coffre-fort** (avancé) : tous les documents produits, organisés,
   chiffrés

**Tier Maître (en plus) :**
9. **Mon parcours VAE** : si applicable au profil
10. **Mes apprenants** : si formateur, suivi de cohorte
11. **Exports Qualiopi** : génération PDF des évaluations pour archivage

Les espaces non déverrouillés restent visibles avec un cadenas et un
"Découvrir le tier supérieur" cliquable (lien Gumroad).

## Pédagogie ancrée dans l'andragogie

Le système prompt d'ANDREA doit incarner les principes de Malcolm Knowles
sur l'andragogie :

1. **Besoin de savoir** : ANDREA explique toujours pourquoi une compétence
   est importante avant de l'enseigner
2. **Concept de soi** : il traite l'apprenant en adulte responsable de son
   apprentissage, pas en élève
3. **Rôle de l'expérience** : il s'appuie sur l'expérience pro de l'apprenant
   comme ressource d'apprentissage
4. **Volonté d'apprendre** : il connecte les apprentissages à des situations
   pro réelles
5. **Orientation de l'apprentissage** : centrée sur la résolution de
   problèmes professionnels concrets
6. **Motivation intrinsèque** : il valorise les progrès, donne du sens

Au-delà de Knowles, ANDREA mobilise aussi :
- Les travaux de **Philippe Carré** sur l'apprenance
- Les apports de **Marcel Lebrun** sur la pédagogie active
- La **taxonomie de Bloom** révisée pour structurer la progression cognitive
- Les principes de **l'évaluation formative** de Scriven et Bloom

ANDREA n'est ni complaisant ni dur : il est **bienveillamment exigeant**.

## Stack technique envisagée (à challenger si tu as mieux)

- **Framework desktop** : Tauri (plus léger qu'Electron, Rust + WebView)
- **LLM local** : Ollama avec Mistral 7B ou Llama 3.2 (quantifié, ~4-5 Go).
  À tester quel modèle gère le mieux la finesse pédagogique en français.
- **STT (speech-to-text)** : Whisper.cpp en local (modèle français)
- **TTS (text-to-speech)** : Piper (voix françaises libres) en priorité,
  fallback sur voix système. Plusieurs voix homme et femme à proposer.
- **RAG** : base vectorielle locale (sqlite-vec ou ChromaDB embarqué)
- **Embeddings** : modèle local (nomic-embed-text ou bge-m3 via Ollama)
- **Frontend** : React + Tailwind + shadcn/ui pour qualité visuelle
  premium (à confirmer selon ton analyse)
- **Stockage local** : SQLite pour profil, progression, historique,
  documents, métadonnées de licence
- **Lecture/évaluation de documents** : support PDF, DOCX, ODT pour que
  l'apprenant puisse soumettre ses productions
- **Crypto** : libsodium ou équivalent Rust pour signature de licence et
  chiffrement des documents
- **Auto-update** : Tauri Updater officiel
- **Distribution** : GitHub Releases ou serveur dédié

## Mon profil technique

- À l'aise avec le terminal, j'ai un Mac (username easy1, /Users/easy1)
- J'aime les solutions itératives, single-file ou self-contained quand possible
- J'ai un développeur disponible pour le backend si besoin de complexité
- Je préfère les UI modernes et propres (pas de design années 2010)
- Je veux comprendre ce que tu fais — explique tes choix au fur et à mesure

## Méthodologie de travail

### Mobilisation systématique de tous tes agents et skills

Pour ce projet premium, je veux que tu mobilises **systématiquement** :

- **Tes sub-agents spécialisés** quand ils sont pertinents (architecte,
  reviewer code, testeur, designer UI, expert sécurité, expert pédagogie
  si disponible, etc.) — n'hésite pas à les solliciter en parallèle
- **Tes skills disponibles** : design frontend, accessibilité (RGAA/WCAG),
  sécurité, performance, tests
- **Une vérification croisée systématique** : après chaque étape de code,
  un agent reviewer doit relire avant que tu me valides "c'est fait"
- **Les meilleures pratiques** : tests automatisés, CI/CD léger,
  documentation inline, gestion d'erreurs propre, logs utiles
- **Le skill frontend-design** spécifiquement pour toute production d'UI

### Méthode par étapes validées

- Tu travailles par étapes. À chaque fin d'étape, tu me montres ce qui
  marche et tu attends mon feu vert avant de passer à la suivante.
- Tu commit régulièrement avec des messages clairs.
- Tu documentes au fur et à mesure dans un README.md.
- Si tu hésites entre deux approches, tu me demandes — tu ne choisis pas seul.
- Tu testes ce que tu codes avant de me dire "c'est fait".

### Qualité visuelle non négociable

L'UI doit être au niveau d'un produit SaaS premium 2026 :
- Design moderne, épuré, cohérent avec l'identité ANDREA
- Mode sombre + mode clair
- Animations subtiles et fluides (pas de surcharge)
- Typographie soignée
- Palette de couleurs réfléchie et signature
- Responsive aux différentes tailles de fenêtre
- Accessible (contrastes, navigation clavier, lecteurs d'écran)

Sollicite ton skill frontend-design pour les choix esthétiques.

## Contrainte d'installation

L'utilisateur final n'est **pas développeur**. L'installation doit être :
double-clic sur un installeur, ça s'installe, ça marche. Pas de terminal,
pas de "il faut d'abord installer X".

Cela signifie qu'**Ollama et le modèle LLM doivent être embarqués ou
téléchargés automatiquement par l'application au premier lancement**, avec
une barre de progression claire et une explication rassurante de ce qui se
passe.

Le premier lancement doit être un moment soigné :
1. Bienvenue à ANDREA (animation, branding)
2. Saisie de la clé de licence (avec lien d'achat Gumroad)
3. Téléchargement du moteur (avec explication "nous installons votre
   formateur, cela ne se fera qu'une fois, environ 5 Go")
4. Onboarding personnalisé (prénom, contexte, objectifs, choix avatar/voix)
5. Première session de découverte guidée par ANDREA lui-même

## Plan de développement attendu

### Phase 1 — ANDREA Découverte (MVP, sortie cible M+3)

**Étape 1 : Fondations techniques**
- Setup projet Tauri + React + Tailwind + shadcn
- Architecture séparation code/données
- Système de licence (génération + validation Ed25519)
- Base SQLite avec versionnement de schéma
- Auto-updater Tauri configuré
- Système d'export/import .andrea-backup

**Étape 2 : Moteur conversationnel local**
- Intégration Ollama embarqué (téléchargement auto premier lancement)
- Choix et benchmark du LLM (Mistral 7B vs Llama 3.2 vs autres) en
  français pour la finesse pédagogique
- STT via Whisper.cpp local
- TTS via Piper avec voix françaises (homme + femme, plusieurs choix)
- Gestion de la conversation (contexte, mémoire courte, barge-in)

**Étape 3 : Référentiel FPA en RAG**
- Récupération du REAC et RE officiels
- Structuration en chunks pédagogiquement cohérents
- Vectorisation et indexation locale (sqlite-vec)
- Tests de rappel sur questions types

**Étape 4 : Système prompt ANDREA Formateur**
- Incarnation du rôle de formateur expert en andragogie
- Garde-fous (pas d'invention de référence, citation des sources)
- Adaptation au profil utilisateur
- Tests pédagogiques avec scénarios réels

**Étape 5 : Onboarding et personnalisation**
- Premier lancement scénarisé
- Saisie clé + profil + objectifs
- Galerie d'avatars (incluant représentation ultramarine)
- Sélection et écoute de voix
- Sauvegarde profil

**Étape 6 : Interface utilisateur Découverte**
- Espace "Mes cours"
- Espace "Ma progression"
- Espace "Mes paramètres"
- Espace "Mon coffre-fort" (basique)
- Espaces verrouillés visibles (cadenas + lien upgrade)
- Mode sombre/clair, accessibilité

**Étape 7 : Packaging et distribution**
- Build cross-platform macOS (priorité) + Windows
- Signature de code (Apple Developer + certificat Windows)
- Installeur DMG (Mac) et MSI (Windows)
- Configuration GitHub Releases pour auto-update
- Tests sur machines réelles diverses

**Étape 8 : Documentation utilisateur**
- Guide de démarrage rapide
- FAQ
- Vidéo de présentation
- Page Gumroad complète

### Phase 2 — ANDREA Pro (sortie cible M+6)

**Étape 9 : Module évaluation de documents**
- Lecture PDF, DOCX, ODT
- Système prompt ANDREA Évaluateur
- Grille d'évaluation calée sur RE officiel
- Restitution structurée des retours
- Sauvegarde des versions et historique d'évaluation

**Étape 10 : Assistant Dossier Professionnel**
- Structure type DP
- Accompagnement étape par étape
- Vérification des preuves de compétences

**Étape 11 : Mises en situation professionnelle**
- Bibliothèque de scénarios MSP
- Mode entraînement
- Évaluation post-MSP

**Étape 12 : Simulateur de jury**
- Simulation entretien technique
- Simulation entretien final
- Retours détaillés

**Étape 13 : Coffre-fort avancé**
- Organisation arborescente des documents
- Chiffrement local
- Versions et historique

**Étape 14 : Tableau de bord progression avancé**
- Vue par compétence
- Indicateur "prêt pour l'épreuve %"
- Recommandations adaptatives

**Étape 15 : Migration v1 → v2**
- Migration automatique de la base de données
- Activation transparente des nouveaux modules par clé Pro
- Tests de migration sans perte de données

### Phase 3 — ANDREA Maître (sortie cible M+9 ou M+12)

À détailler ultérieurement selon retours marché v1 et v2.

## Pour chaque étape, tu dois préciser

- Livrables concrets
- Critères de validation
- Durée estimée
- Sub-agents/skills mobilisés
- Risques identifiés

## Risques juridiques à anticiper

- "Titre Professionnel" et "FPA" sont des appellations protégées par le
  Ministère du Travail — mention obligatoire "produit indépendant non
  affilié au Ministère du Travail ni à l'AFPA"
- Limite de responsabilité : ANDREA prépare mais ne garantit pas la
  réussite à l'épreuve
- RGPD : 100% local donc largement conforme, mais à documenter
- Conditions d'utilisation et licence à rédiger
- Mention de la propriété intellectuelle de Julien PERROT sur le contenu
  pédagogique
