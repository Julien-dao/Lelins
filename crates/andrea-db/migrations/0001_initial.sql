-- Migration 0001 — initial schema for ANDREA Discovery (and forward-compatible
-- with Pro/Master tiers, since per architecture decision all tables exist
-- from v1).
--
-- This migration is wrapped in a single transaction by the runner.
-- Idempotency: every CREATE uses IF NOT EXISTS where supported.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS _metadata (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT OR IGNORE INTO _metadata (key, value) VALUES ('schema_version', '1');
INSERT OR IGNORE INTO _metadata (key, value) VALUES ('app_version_first_install', '0.1.0');
INSERT OR IGNORE INTO _metadata (key, value) VALUES ('created_at', strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

-- Singleton user profile (id is checked = 1 to enforce uniqueness).
CREATE TABLE IF NOT EXISTS profile (
    id              INTEGER PRIMARY KEY CHECK (id = 1),
    prenom          TEXT NOT NULL,
    appel           TEXT NOT NULL,
    email           TEXT NOT NULL,
    contexte        TEXT NOT NULL,
    niveau_depart   TEXT NOT NULL,
    objectif        TEXT NOT NULL,
    cadence         TEXT NOT NULL,
    date_epreuve    TEXT,
    tutoiement      INTEGER NOT NULL DEFAULT 0 CHECK (tutoiement IN (0, 1)),
    voix_id         TEXT,
    avatar_id       TEXT,
    formateur_nom   TEXT NOT NULL DEFAULT 'ANDREA',
    langue          TEXT NOT NULL DEFAULT 'fr',
    theme           TEXT NOT NULL DEFAULT 'system' CHECK (theme IN ('system', 'dark', 'light')),
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS license (
    id               INTEGER PRIMARY KEY CHECK (id = 1),
    raw_key          TEXT NOT NULL,
    tier             TEXT NOT NULL CHECK (tier IN ('DECO', 'PRO', 'MAIT', 'BNDL')),
    email_hash       BLOB NOT NULL,
    features_bitmask INTEGER NOT NULL DEFAULT 0,
    validated_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    history_json     TEXT NOT NULL DEFAULT '[]'
);

-- Reference: the 13 CPs of REAC V07 (preloaded by the application on first run,
-- not in this migration to keep migrations purely structural).
CREATE TABLE IF NOT EXISTS competence (
    code         TEXT PRIMARY KEY,
    ccp          TEXT NOT NULL,
    ordre        INTEGER NOT NULL,
    intitule     TEXT NOT NULL,
    description  TEXT NOT NULL,
    source_ref   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS progression (
    competence_code TEXT PRIMARY KEY REFERENCES competence(code) ON DELETE CASCADE,
    niveau          INTEGER NOT NULL DEFAULT 0 CHECK (niveau BETWEEN 0 AND 4),
    evidence_count  INTEGER NOT NULL DEFAULT 0,
    notes_md        TEXT,
    last_seen_at    TEXT,
    updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS session (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ended_at    TEXT,
    mode        TEXT NOT NULL CHECK (mode IN ('voice', 'text', 'mixed')),
    competences TEXT,
    summary_md  TEXT,
    rating      INTEGER CHECK (rating IS NULL OR rating BETWEEN 1 AND 5)
);

CREATE TABLE IF NOT EXISTS turn (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id      INTEGER NOT NULL REFERENCES session(id) ON DELETE CASCADE,
    idx             INTEGER NOT NULL,
    role            TEXT NOT NULL CHECK (role IN ('user', 'andrea', 'system')),
    content_md      TEXT NOT NULL,
    audio_path      TEXT,
    citations_json  TEXT,
    tokens_in       INTEGER,
    tokens_out      INTEGER,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (session_id, idx)
);

CREATE INDEX IF NOT EXISTS idx_turn_session ON turn(session_id);

-- Documents are gated behind tier=Pro at the application level, but the
-- table exists from v1 to avoid migrations on tier upgrade.
CREATE TABLE IF NOT EXISTS document (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    type           TEXT NOT NULL CHECK (type IN ('dp', 'support', 'note', 'autre')),
    ccp            TEXT,
    filename       TEXT NOT NULL,
    path_encrypted TEXT NOT NULL,
    sha256         BLOB NOT NULL,
    size_bytes     INTEGER NOT NULL CHECK (size_bytes >= 0),
    status         TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'review', 'finalized')),
    uploaded_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS evaluation (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id  INTEGER NOT NULL REFERENCES document(id) ON DELETE CASCADE,
    grille       TEXT NOT NULL CHECK (grille IN ('jury', 'formative', 'sommative')),
    score_global INTEGER,
    feedback_md  TEXT NOT NULL,
    axes_md      TEXT,
    evaluated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Audit / debug events.
CREATE TABLE IF NOT EXISTS event (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    type         TEXT NOT NULL,
    payload_json TEXT,
    occurred_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS backup (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    filename       TEXT NOT NULL,
    schema_version INTEGER NOT NULL,
    app_version    TEXT NOT NULL,
    size_bytes     INTEGER NOT NULL CHECK (size_bytes >= 0),
    sha256         BLOB NOT NULL,
    exported_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
