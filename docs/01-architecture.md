# 01 — Architecture détaillée d'ANDREA

> Document produit le 6 mai 2026, à valider avant attaque de la phase 1.

---

## 1. Vue d'ensemble

ANDREA est une **application desktop offline**, monolithique côté front, à
couches métier claires côté back, avec **trois sidecars** (Ollama, Whisper,
Piper) lancés et supervisés par le runtime Tauri. Toutes les données utilisateur
sont stockées dans un dossier système hors du dossier d'installation.

```
┌──────────────────────────────────────────────────────────────────┐
│                         Utilisateur                              │
└──────────────────────────────────────────────────────────────────┘
                              │
┌──────────────────────────────────────────────────────────────────┐
│  Tauri WebView (React + Tailwind + shadcn + Framer Motion)       │
│   ─ Onboarding · Mes cours · Ma progression · Coffre-fort        │
│   ─ Paramètres · Espaces verrouillés (Pro / Maître)              │
└──────────────────────────────────────────────────────────────────┘
                              │ IPC (commands + events)
┌──────────────────────────────────────────────────────────────────┐
│  Tauri Core (Rust)                                                │
│   ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌──────────────┐  │
│   │  License  │  │    DB     │  │    LLM    │  │     RAG      │  │
│   │ (Ed25519) │  │ (SQLite + │  │ (ollama-  │  │ (sqlite-vec  │  │
│   │           │  │ migrations│  │  rs API   │  │  + bge-m3)   │  │
│   │           │  │ rusqlite) │  │  HTTP)    │  │              │  │
│   └───────────┘  └───────────┘  └───────────┘  └──────────────┘  │
│   ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌──────────────┐  │
│   │   STT     │  │   TTS     │  │   Docs    │  │   Backup &   │  │
│   │ (whisper- │  │  (Piper   │  │ (PDF/DOCX │  │   Crypto     │  │
│   │  rs local)│  │  sidecar) │  │  /ODT)    │  │ (libsodium)  │  │
│   └───────────┘  └───────────┘  └───────────┘  └──────────────┘  │
└──────────────────────────────────────────────────────────────────┘
                              │ subprocess + HTTP localhost
┌──────────────────────────────────────────────────────────────────┐
│  Sidecars                                                         │
│   ─ ollama serve  (port 11434, modèle FR)                        │
│   ─ piper voice service (stdin → stdout audio)                   │
│   ─ whisper.cpp (intégré via whisper-rs, pas un sidecar séparé)  │
└──────────────────────────────────────────────────────────────────┘
                              │ filesystem
┌──────────────────────────────────────────────────────────────────┐
│  Données utilisateur (hors dossier d'install)                     │
│   ─ ~/Library/Application Support/ANDREA/   (macOS)              │
│   ─ %APPDATA%\ANDREA\                       (Windows)            │
│       ├─ andrea.db          SQLite (profil, sessions, progression)│
│       ├─ documents/         Documents apprenant chiffrés          │
│       ├─ models/            Ollama models (gérés par Ollama)      │
│       ├─ whisper/           Modèles GGML Whisper                  │
│       ├─ piper/             Voix ONNX Piper                       │
│       ├─ rag/               Index RAG du référentiel              │
│       └─ logs/               Logs rotatifs                        │
└──────────────────────────────────────────────────────────────────┘
```

---

## 2. Arborescence du projet

```
andrea/
├── BRIEF.md                          # Brief produit, source de vérité
├── README.md                         # Documentation projet
├── docs/                             # Documents stratégie & ADRs
├── .github/workflows/                # CI : test, build, release, sign, notarize
│
├── apps/
│   ├── desktop/                      # Application Tauri principale
│   │   ├── src-tauri/                # Backend Rust
│   │   │   ├── Cargo.toml
│   │   │   ├── tauri.conf.json
│   │   │   ├── build.rs
│   │   │   ├── icons/
│   │   │   ├── resources/            # Bundlés dans l'app
│   │   │   │   ├── prompts/          # Versions des system prompts
│   │   │   │   ├── referentiel/      # JSON pré-chunked du référentiel
│   │   │   │   └── avatars/          # Galerie de portraits
│   │   │   ├── sidecars/             # Binaires natifs par target-triple
│   │   │   │   ├── ollama-aarch64-apple-darwin
│   │   │   │   ├── ollama-x86_64-apple-darwin
│   │   │   │   ├── ollama-x86_64-pc-windows-msvc.exe
│   │   │   │   └── piper-* (idem)
│   │   │   ├── migrations/           # SQL versionné, en ordre lexicographique
│   │   │   │   ├── 0001_initial.sql
│   │   │   │   ├── 0002_add_evaluations.sql  (sera ajouté en v2)
│   │   │   │   └── ...
│   │   │   └── src/
│   │   │       ├── main.rs
│   │   │       ├── lib.rs
│   │   │       ├── error.rs          # AndreaError + From impls
│   │   │       ├── config.rs         # Paths système, version
│   │   │       ├── commands/         # #[tauri::command] exposés au front
│   │   │       │   ├── onboarding.rs
│   │   │       │   ├── conversation.rs
│   │   │       │   ├── progression.rs
│   │   │       │   ├── documents.rs
│   │   │       │   ├── settings.rs
│   │   │       │   └── backup.rs
│   │   │       ├── domain/           # Modèles métier purs
│   │   │       │   ├── profile.rs
│   │   │       │   ├── progression.rs
│   │   │       │   ├── session.rs
│   │   │       │   ├── document.rs
│   │   │       │   └── tier.rs
│   │   │       ├── license/          # Validation Ed25519
│   │   │       │   ├── mod.rs
│   │   │       │   ├── format.rs     # Encode/decode Crockford
│   │   │       │   ├── verifier.rs   # Verify Ed25519
│   │   │       │   └── tier.rs       # Mapping tier → features
│   │   │       ├── db/               # Couche persistence
│   │   │       │   ├── mod.rs
│   │   │       │   ├── pool.rs       # rusqlite + r2d2
│   │   │       │   ├── migrations.rs # Runner maison idempotent
│   │   │       │   └── repos/        # Repository pattern
│   │   │       ├── llm/              # Trait LlmProvider + impl Ollama
│   │   │       │   ├── mod.rs
│   │   │       │   ├── ollama.rs
│   │   │       │   ├── prompt.rs     # Construction prompt template
│   │   │       │   └── streaming.rs
│   │   │       ├── stt/              # Whisper-rs
│   │   │       ├── tts/              # Piper sidecar driver
│   │   │       ├── rag/              # sqlite-vec + ingestion + retrieval
│   │   │       │   ├── mod.rs
│   │   │       │   ├── embed.rs      # Appelle Ollama embed (bge-m3)
│   │   │       │   ├── index.rs      # Insert / query sqlite-vec
│   │   │       │   └── ingest.rs     # Chunk + index referentiel/
│   │   │       ├── docs/             # PDF/DOCX/ODT
│   │   │       ├── crypto/           # libsodium (xchacha20poly1305) + argon2id
│   │   │       ├── backup/           # Export/import .andrea-backup
│   │   │       ├── sidecar/          # Lifecycle des sidecars
│   │   │       │   ├── mod.rs
│   │   │       │   ├── ollama.rs     # spawn, health-check, restart
│   │   │       │   └── piper.rs
│   │   │       └── updater/          # Hooks tauri-plugin-updater
│   │   ├── src/                      # Frontend React
│   │   │   ├── main.tsx
│   │   │   ├── App.tsx
│   │   │   ├── routes/               # File-based ou react-router
│   │   │   │   ├── onboarding/
│   │   │   │   ├── courses/          # Mes cours
│   │   │   │   ├── progression/      # Ma progression
│   │   │   │   ├── vault/            # Mon coffre-fort
│   │   │   │   ├── settings/         # Mes paramètres
│   │   │   │   └── locked/           # Espaces verrouillés (cadenas)
│   │   │   ├── components/
│   │   │   │   ├── ui/               # shadcn imports
│   │   │   │   ├── andrea/           # Avatar animé, indicateurs d'état
│   │   │   │   ├── chat/             # Bulles conversation, input vocal/texte
│   │   │   │   └── layout/
│   │   │   ├── features/             # Slices logique
│   │   │   ├── hooks/
│   │   │   │   ├── useTier.ts        # Hook gate tier
│   │   │   │   ├── useAndreaVoice.ts
│   │   │   │   └── ...
│   │   │   ├── lib/
│   │   │   │   ├── ipc.ts            # Wrapper invoke + types
│   │   │   │   ├── tier-gate.ts
│   │   │   │   └── format.ts
│   │   │   ├── stores/               # Zustand ou jotai (à choisir)
│   │   │   └── styles/
│   │   ├── public/
│   │   ├── package.json
│   │   ├── tailwind.config.ts
│   │   └── vite.config.ts
│   │
│   └── license-server/               # Microservice génération clés
│       ├── src/
│       │   ├── webhook.ts            # Webhook Gumroad → générer clé
│       │   ├── generate.ts           # Génération Ed25519
│       │   └── revoke.ts             # Révocation
│       ├── package.json
│       └── README.md
│
├── packages/
│   ├── prompts/                      # Versions versionnées des system prompts
│   │   ├── andrea-formateur-v1.md
│   │   ├── andrea-evaluateur-v1.md   # (Pro v2)
│   │   └── andrea-guide-v1.md        # (Pro v2)
│   └── shared-types/                 # Types Rust → TS via ts-rs
│
├── data/                             # Sources brutes (gitignored partiellement)
│   ├── referentiel-fpa/              # PDF officiels REAC/RE
│   │   ├── reac-v07-2022-12-21.pdf
│   │   ├── re-v07-2022-12-21.pdf
│   │   └── arretes/
│   ├── chunked/                      # JSONL pré-chunked, embarqué dans app
│   └── voix-samples/
│
├── scripts/
│   ├── chunk-referentiel.ts          # Pipeline de chunking
│   ├── download-models.sh            # Bootstrap modèles dev
│   └── verify-license.ts             # CLI test verifier
│
└── tests/
    ├── e2e/                           # Playwright sur Tauri
    └── fixtures/                      # Clés de test, exemples DP
```

---

## 3. Modèle de données SQLite

### Tables (v1 Découverte — toutes prévues dès le départ)

```sql
-- Métadonnées de schéma (versionnement migrations)
CREATE TABLE _metadata (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
-- INSERT INTO _metadata VALUES ('schema_version', '1');
-- INSERT INTO _metadata VALUES ('created_at', '...');
-- INSERT INTO _metadata VALUES ('andrea_version_first_install', '1.0.0');

-- Profil utilisateur (un seul, mais structure extensible)
CREATE TABLE profile (
  id              INTEGER PRIMARY KEY CHECK (id = 1),  -- singleton
  prenom          TEXT NOT NULL,
  appel           TEXT NOT NULL,                       -- comment être appelé
  email           TEXT NOT NULL,                       -- saisi à l'install
  contexte        TEXT NOT NULL,                       -- enum string
  niveau_depart   TEXT NOT NULL,
  objectif        TEXT NOT NULL,                       -- titre|ccp1|ccp2|...|montee
  cadence         TEXT NOT NULL,                       -- intensif|regulier|libre
  date_epreuve    DATE,
  tutoiement      INTEGER NOT NULL DEFAULT 0,          -- 0=vous, 1=tu
  voix_id         TEXT,
  avatar_id       TEXT,
  formateur_nom   TEXT NOT NULL DEFAULT 'ANDREA',
  langue          TEXT NOT NULL DEFAULT 'fr',
  theme           TEXT NOT NULL DEFAULT 'system',      -- system|dark|light
  created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Licence active
CREATE TABLE license (
  id              INTEGER PRIMARY KEY CHECK (id = 1),  -- singleton
  raw_key         TEXT NOT NULL,
  tier            TEXT NOT NULL,                       -- DECO|PRO|MAITRE
  email_hash      BLOB NOT NULL,
  features_bitmask INTEGER NOT NULL DEFAULT 0,
  validated_at    TIMESTAMP NOT NULL,
  history_json    TEXT NOT NULL DEFAULT '[]'           -- log des changements
);

-- Référentiel : les 13 CP du REAC V07 (pré-chargées au premier lancement)
CREATE TABLE competence (
  code            TEXT PRIMARY KEY,                    -- CP1..CP13
  ccp             TEXT NOT NULL,                       -- CCP1..CCP4
  ordre           INTEGER NOT NULL,
  intitule        TEXT NOT NULL,
  description     TEXT NOT NULL,
  source_ref      TEXT NOT NULL                        -- "REAC V07 21/12/2022"
);

-- Progression de l'apprenant par compétence
CREATE TABLE progression (
  competence_code TEXT PRIMARY KEY REFERENCES competence(code),
  niveau          INTEGER NOT NULL DEFAULT 0,          -- 0..4 (Bloom révisée)
  evidence_count  INTEGER NOT NULL DEFAULT 0,          -- # tours où abordée
  notes_md        TEXT,
  last_seen_at    TIMESTAMP,
  updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Sessions de conversation
CREATE TABLE session (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  started_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  ended_at        TIMESTAMP,
  mode            TEXT NOT NULL,                       -- voice|text|mixed
  competences     TEXT,                                -- JSON [CP3, CP5]
  summary_md      TEXT,                                -- généré post-session
  rating          INTEGER                              -- ressenti utilisateur
);

-- Tours de conversation
CREATE TABLE turn (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id      INTEGER NOT NULL REFERENCES session(id) ON DELETE CASCADE,
  idx             INTEGER NOT NULL,
  role            TEXT NOT NULL,                       -- user|andrea|system
  content_md      TEXT NOT NULL,
  audio_path      TEXT,                                -- relative path
  citations_json  TEXT,                                -- chunks RAG cités
  tokens_in       INTEGER,
  tokens_out      INTEGER,
  created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (session_id, idx)
);

-- Documents apprenant (DP, supports, productions) — Pro mais structure dès v1
CREATE TABLE document (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  type            TEXT NOT NULL,                       -- dp|support|note|autre
  ccp             TEXT,                                -- rattachement éventuel
  filename        TEXT NOT NULL,
  path_encrypted  TEXT NOT NULL,                       -- relative
  sha256          BLOB NOT NULL,
  size_bytes      INTEGER NOT NULL,
  status          TEXT NOT NULL DEFAULT 'draft',       -- draft|review|finalized
  uploaded_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Évaluations (Pro mais structure dès v1)
CREATE TABLE evaluation (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  document_id     INTEGER NOT NULL REFERENCES document(id) ON DELETE CASCADE,
  grille          TEXT NOT NULL,                       -- jury|formative|sommative
  score_global    INTEGER,
  feedback_md     TEXT NOT NULL,
  axes_md         TEXT,
  evaluated_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index RAG (sqlite-vec virtual table)
CREATE VIRTUAL TABLE rag_chunks USING vec0(
  chunk_id        TEXT PRIMARY KEY,
  embedding       FLOAT[1024]                          -- bge-m3 dim
);
CREATE TABLE rag_chunks_meta (
  chunk_id        TEXT PRIMARY KEY,
  source_doc      TEXT NOT NULL,
  source_section  TEXT,
  citation        TEXT NOT NULL,                       -- "REAC V07, CCP2, CP4"
  text            TEXT NOT NULL
);

-- Audit / debug
CREATE TABLE event (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  type            TEXT NOT NULL,
  payload_json    TEXT,
  occurred_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Backups produits localement
CREATE TABLE backup (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  filename        TEXT NOT NULL,
  schema_version  INTEGER NOT NULL,
  app_version     TEXT NOT NULL,
  size_bytes      INTEGER NOT NULL,
  sha256          BLOB NOT NULL,
  exported_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Pourquoi tout dès v1 ?** Pour qu'une migration v1→v2 soit triviale (les tables
existent déjà, on ajoute juste les nouveaux usages côté code). Pas de migration
de données entre tiers. Seules les **migrations entre versions d'app** modifient
le schéma (ajout de colonnes, de tables).

---

## 4. Système de licence Ed25519

### Format de clé (proposition)

```
ANDREA-DECO-XXXXX-XXXXX-XXXXX-XXXXX
       │     └─────────────────────┘
       │     20 chars Base32 Crockford = 100 bits utiles
       │
       └─ Tier (DECO | PRO | MAIT) — 4 chars fixes
```

**Payload encodé dans les 100 bits :**

| Champ | Bits | Contenu |
|---|---|---|
| Version format | 4 | 0x1 |
| Tier | 4 | 0=DECO, 1=PRO, 2=MAITRE, 3=BUNDLE, … |
| Email hash tronqué | 32 | `SHA256(email)[0..4]` |
| Date émission | 16 | jours depuis 2026-01-01 |
| Features bitmask | 16 | flags fines (cohorte, VAE, …) |
| **MAC tronqué** | **28** | `BLAKE2b(server_secret \|\| payload)[0..28 bits]` |

**Sécurité réelle :** ~28 bits de MAC = forge brute-forçable en quelques heures
si l'attaquant a un seul exemplaire du serveur de validation. Mais :

1. La clé secrète serveur n'est jamais distribuée.
2. Le hash email lie la clé à un acheteur spécifique : un crack n'aide pas à
   piratage de masse, juste à un crack individuel.
3. Le modèle de menace est **dissuasion contre le partage occasionnel**, pas
   inviolabilité à la JetBrains.

### Alternative envisagée : signature Ed25519 réelle

Une signature Ed25519 valide fait 512 bits = 103 chars Base32. Trop long pour
être collé/saisi à la main par un acheteur Gumroad. Solution rejetée.

### Génération côté serveur

Microservice `apps/license-server/` (Node.js + Hono ou Rust + Axum) :

```
POST /webhook/gumroad
  → vérifie HMAC Gumroad
  → produit_id → tier
  → génère payload (4+4+32+16+16 = 72 bits) + MAC 28 bits
  → encode Base32 Crockford, formate "ANDREA-TIER-XXXXX-..."
  → email à l'acheteur via SMTP
  → log dans une SQLite licenses.db (clé, email, tier, date, status)

POST /admin/revoke {key}
  → marque la clé révoquée
  → ajoute à la blocklist embarquée dans le prochain release
```

### Validation côté app (offline)

```rust
fn verify(raw: &str, server_pubkey_hash: &[u8]) -> Result<License> {
    let normalized = normalize_crockford(raw)?;
    let (tier_str, payload_b32) = parse_format(&normalized)?;
    let bits = decode_crockford(payload_b32)?;
    let payload = bits[..72];
    let mac = bits[72..100];

    // MAC est calculé avec une clé secrète embarquée dans le binaire
    // (offuscation light, pas inviolabilité)
    let expected = blake2b(SERVER_SECRET, payload)[..28];
    if !constant_time_eq(mac, expected) {
        return Err(InvalidLicense);
    }

    // Vérifier blocklist locale
    if BLOCKLIST.contains(&raw) {
        return Err(RevokedLicense);
    }

    // Comparer hash email
    let email_hash = sha256(profile.email.lowercase())[0..4];
    if email_hash != payload.email_hash {
        return Err(EmailMismatch);
    }

    Ok(License { tier, features, ... })
}
```

### Upgrade entre tiers

```
DECO → PRO :
  - utilisateur paie 100 € sur Gumroad (produit "upgrade DECO→PRO" ou code promo)
  - reçoit nouvelle clé ANDREA-PRO-...
  - saisit dans Paramètres → Licence
  - validation, mise à jour table license, history_json append, déverrouillage immédiat
  - aucune migration de données
```

---

## 5. Migrations BDD entre versions d'app

### Approche maison minimaliste

```rust
// db/migrations.rs
const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_initial", include_str!("../migrations/0001_initial.sql")),
    ("0002_add_x",    include_str!("../migrations/0002_add_x.sql")),
    // ...
];

pub fn run_migrations(conn: &mut Connection, on_progress: impl Fn(MigrationProgress)) -> Result<()> {
    let current = read_schema_version(conn)?;
    let target  = MIGRATIONS.len() as u32;

    if current == target {
        return Ok(());
    }

    // Backup automatique avant migration
    let backup_path = backup_before_migration(conn)?;
    on_progress(MigrationProgress::BackupCreated(backup_path));

    let tx = conn.transaction()?;
    for (i, (name, sql)) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        on_progress(MigrationProgress::Running(name));
        tx.execute_batch(sql)?;
        tx.execute(
            "UPDATE _metadata SET value = ?1 WHERE key = 'schema_version'",
            params![(i + 1) as i64],
        )?;
    }
    tx.commit()?;

    on_progress(MigrationProgress::Complete(target));
    Ok(())
}
```

**Garanties :**
- Idempotent (rejouer une migration ne casse rien — chaque SQL utilise
  `CREATE TABLE IF NOT EXISTS`, `ALTER TABLE ADD COLUMN` après check, etc.)
- Transactionnel (tout ou rien)
- Backup auto avant migration → rollback possible
- Tests CI : matrix de upgrades from N to N+1 sur fixtures

### Alternatives évaluées

- **refinery** : OK, mais override de structure imposé.
- **sqlx_migrate** : nécessite sqlx, plus lourd que rusqlite que je préfère pour
  load_extension (sqlite-vec).
- **Diesel migrations** : trop d'abstraction, ORM non nécessaire ici.

→ **Maison choisi pour simplicité et contrôle.**

---

## 6. Flux conversationnel détaillé

### "L'apprenant parle → ANDREA répond"

```
[1] UI bouton micro pressé
    └─ invoke('start_listening')

[2] Backend démarre capture audio (cpal stream 16 kHz mono)
    └─ event('listening_started')

[3] VAD (silero ou WebRTC VAD) détecte fin de phrase ou silence prolongé
    └─ stop capture, buffer WAV en mémoire

[4] STT local : whisper-rs.transcribe(buffer, lang=fr, model=small)
    └─ event('user_transcript', text)
    └─ insert turn(role=user, content_md=text)

[5] Construction du contexte
    ├─ system prompt (chargé depuis resources/prompts/andrea-formateur-v1.md)
    │   avec interpolation {prenom}, {tutoiement}, etc.
    ├─ derniers N tours (token-budget : ~2000 tokens)
    └─ RAG : embed(text) via Ollama bge-m3 → top-K chunks référentiel via sqlite-vec

[6] Appel Ollama streaming
    └─ ollama-rs.generate_stream(model=mistral-small3.2, prompt, options={ temperature: 0.3, ... })
    └─ chaque chunk reçu :
        - bufferise jusqu'à frontière phrase (./?/!)
        - event('andrea_partial', partial_text)
        - dispatch vers Piper

[7] TTS Piper (sidecar long-running, stdin/stdout)
    └─ par phrase : write phrase → read PCM → stream vers buffer audio
    └─ frontend joue le buffer à mesure (audio queue)
    └─ event('andrea_speaking', true)

[8] Fin de génération
    └─ insert turn(role=andrea, content_md=full_response, citations_json)
    └─ event('andrea_done', { full_text, citations })
    └─ analyse post-réponse async :
        - mise à jour progression sur les CP abordés
        - update last_seen_at de session

[9] Barge-in
    └─ si VAD détecte voix utilisateur pendant TTS :
        - kill Piper output
        - stop audio queue
        - retour étape [2]
```

### Mode texte

Identique sans étapes [2-3] et [7]. L'utilisateur peut basculer texte/vocal à
tout moment.

### Reprise de session

`session.summary_md` est généré à la fin de chaque session par un appel LLM
court. Au début de la session suivante, ce résumé est injecté dans le contexte
(« Lors de notre précédent échange, nous avions abordé… ») pour continuité
pédagogique.

---

## 7. Architecture des espaces et gating tier

### Côté front

```ts
// lib/tier-gate.ts
export const SPACES = [
  { id: 'courses',     tier: 'DECO',   route: '/courses' },
  { id: 'progression', tier: 'DECO',   route: '/progression' },
  { id: 'vault',       tier: 'DECO',   route: '/vault' },
  { id: 'settings',    tier: 'DECO',   route: '/settings' },
  { id: 'dp',          tier: 'PRO',    route: '/dp' },
  { id: 'msp',         tier: 'PRO',    route: '/msp' },
  { id: 'jury',        tier: 'PRO',    route: '/jury' },
  { id: 'cohort',      tier: 'MAITRE', route: '/cohort' },
  { id: 'qualiopi',    tier: 'MAITRE', route: '/qualiopi' },
  { id: 'scenarios',   tier: 'MAITRE', route: '/scenarios' },
] as const;

const TIER_RANK = { DECO: 0, PRO: 1, MAITRE: 2 };
export function isUnlocked(spaceTier: Tier, userTier: Tier) {
  return TIER_RANK[userTier] >= TIER_RANK[spaceTier];
}
```

```tsx
// composant ProtectedSpace
function ProtectedSpace({ id, children }: Props) {
  const { tier, isUnlocked, upgradeUrl } = useTier();
  const space = SPACES.find(s => s.id === id)!;
  if (!isUnlocked(space.tier)) {
    return <LockedSpaceCTA target={space.tier} url={upgradeUrl(space.tier)} />;
  }
  return children;
}
```

Les espaces verrouillés affichent un cadenas + CTA Gumroad mais sont visibles
dans la navigation (incite à l'upgrade).

### Côté back

Toutes les commandes `#[tauri::command]` qui touchent à un module Pro/Maître
vérifient le tier en début :

```rust
#[tauri::command]
async fn evaluate_document(state: State<AppState>, doc_id: i64) -> Result<Evaluation> {
    require_tier(state.license_tier(), Tier::Pro)?;
    // ...
}
```

Double gate (UI + back) pour éviter qu'une CLI custom ne contourne.

---

## 8. Export / import `.andrea-backup`

### Format

```
.andrea-backup (binaire)
├─ magic: "ANDR" (4 octets)
├─ version: u8
├─ encryption: u8 (0=auto-derived, 1=passphrase)
├─ kdf_salt: 16 octets (si passphrase)
├─ nonce: 24 octets (XChaCha20)
├─ ciphertext: tar.zstd chiffré
│   ├─ andrea.db (snapshot SQLite)
│   ├─ documents/ (fichiers chiffrés tels quels)
│   ├─ manifest.json (metadata, schema_version, app_version)
│   └─ checksums.json
└─ poly1305 tag: 16 octets
```

### Dérivation de clé

- **Mode auto (par défaut)** : clé dérivée de
  `Argon2id(profile.email + machine_id_partial + "ANDREA_BACKUP_V1")`. Permet
  l'import sur la même machine sans passphrase, mais migration sur autre machine
  nécessite la même installation ANDREA + même email.
- **Mode passphrase (avancé)** : Argon2id de la passphrase utilisateur ;
  portable n'importe où.

### Import

- Détection magic + version
- Vérif checksum
- Déchiffrement
- Si `schema_version` < actuelle : exécute migrations automatiques sur le snapshot
- Import en base + dossier documents
- Notification utilisateur

---

## 9. Configuration Tauri Updater

```json
// tauri.conf.json (extrait)
{
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://github.com/julien-dao/andrea/releases/latest/download/latest.json"
      ],
      "dialog": false,
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlz...="
    }
  }
}
```

UX : ANDREA notifie discrètement (toast en bas à droite) quand une mise à jour
est dispo, propose "Installer maintenant" / "Plus tard". Update de sécurité :
notification plus insistante.

---

## 10. Choix techniques justifiés vs alternatives

| Choix | Pourquoi | Alternative écartée | Pourquoi écartée |
|---|---|---|---|
| **Tauri 2.6** | Léger, Rust, écosystème mature 2026 | Electron | 80-200 Mo vs 2-10 Mo, RAM 2-3× |
| **rusqlite + r2d2** | Charge sqlite-vec extension | tauri-plugin-sql (sqlx) | Issue #2622 : pas de `load_extension` |
| **Sidecar Ollama** | MIT, écosystème, modèles | mistral.rs intégré | Compile-time long, complexité Metal/CUDA |
| **Whisper-rs 0.16** | Maintenu, repo migré Codeberg | faster-whisper Python | Embarquement Python = lourd |
| **Piper OHF-Voice GPL-3.0** | Voix françaises libres | XTTS Coqui | Coqui fermé déc. 2025, licence non-commerciale |
| **bge-m3 embeddings** | Multilingue, contexte 8192 | nomic-embed | bge-m3 mieux noté en français |
| **sqlite-vec 0.1.9** | Pure C, multi-OS, simple | sqlite-vss | Déprécié, dépendances Faiss |
| **ed25519-dalek + MAC tronqué** | Réalisme : clé courte saisissable | Ed25519 pure (signature 103 chars) | Inacceptable UX |
| **React + Tailwind + shadcn** | Productivité + qualité visuelle | SolidJS / Svelte | Écosystème shadcn unique en 2026 |
| **Zustand** (à valider) | Léger, simple | Redux Toolkit | Trop de cérémonie ici |
| **Framer Motion** | Animations premium | CSS pur | Avatar animé difficile en CSS pur |
| **Migrations maison Rust** | Contrôle total, light | refinery | Override de structure imposé |
| **GitHub Releases pour update** | Gratuit, fiable | CrabNebula Cloud | Service tiers payant |

---

## 11. Points de risque architecturaux

1. **Ollama sidecar bloqué par sandbox macOS** : tester en CI sur runner macOS
   notarized, pas seulement en dev local.
2. **sqlite-vec extension non signée** : signer `.dylib`/`.dll` avec le même
   Developer ID que l'app, sinon Gatekeeper peut bloquer.
3. **Modèle LLM 24B téléchargé en background** : prévoir reprise sur coupure
   réseau, vérification SHA256, espace disque check préalable.
4. **Migration BDD échoue à mi-parcours** : backup automatique + rollback
   transactionnel + UI de récupération.
5. **Clé licence collée avec espaces/casse foireuse** : normalisation Crockford
   stricte (suppression I/L/O confondus avec 1/0).
6. **Saisie utilisateur dans `.andrea-backup` exposée** : chiffrement systématique,
   jamais de fallback "clear si clé manquante".
7. **Couplage trop fort à Ollama** : abstraction `LlmProvider` trait pour
   permettre bascule mistral.rs si Ollama disparaît.
