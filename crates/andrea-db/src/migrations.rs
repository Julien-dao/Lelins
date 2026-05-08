//! Migration runner.
//!
//! Migrations are stored as SQL files in `migrations/` and embedded at
//! compile time. They are applied in order, transactionally, with the
//! `_metadata.schema_version` row updated at each step.
//!
//! Idempotency contract: every migration must be safely re-runnable, i.e.
//! use `CREATE TABLE IF NOT EXISTS`, `INSERT OR IGNORE`, etc., or wrap its
//! changes behind a `DO NOTHING` guard. The runner's contract is that it
//! will not re-run a migration whose name is already recorded.

use rusqlite::{Connection, OptionalExtension};
use thiserror::Error;

/// All migrations in order. To add a new migration, append a tuple here
/// and create the corresponding file in `migrations/`.
pub const MIGRATIONS: &[(&str, &str)] = &[(
    "0001_initial",
    include_str!("../migrations/0001_initial.sql"),
)];

/// Latest schema version supported by this crate.
pub const fn current_target() -> u32 {
    MIGRATIONS.len() as u32
}

/// Errors that may occur while running migrations.
#[derive(Debug, Error)]
pub enum MigrationsError {
    /// SQLite error from the underlying connection.
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    /// The recorded schema version is newer than what this binary supports
    /// (i.e. the user opened a DB written by a future ANDREA version).
    #[error("database schema version {found} is newer than supported {supported}")]
    FutureSchema {
        /// Version recorded in the DB.
        found: u32,
        /// Maximum version this binary can apply.
        supported: u32,
    },
    /// A migration script failed.
    #[error("migration `{name}` failed: {source}")]
    StepFailed {
        /// Name of the migration that failed.
        name: &'static str,
        /// Underlying SQLite error.
        #[source]
        source: rusqlite::Error,
    },
}

/// Progress events emitted by the migration runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationProgress {
    /// The DB is already up to date — nothing to do.
    AlreadyUpToDate {
        /// Version recorded in the DB.
        version: u32,
    },
    /// About to begin migrations.
    Starting {
        /// Version currently recorded.
        from: u32,
        /// Target version after running all pending migrations.
        to: u32,
    },
    /// A specific migration step is running.
    Step {
        /// Migration index (1-based).
        index: u32,
        /// Migration name.
        name: &'static str,
    },
    /// Migrations completed successfully.
    Complete {
        /// Final schema version.
        version: u32,
    },
}

/// Read the current schema version from `_metadata`. Returns `0` if the
/// `_metadata` table does not yet exist (fresh DB).
pub fn schema_version(conn: &Connection) -> Result<u32, rusqlite::Error> {
    // Check that _metadata exists at all.
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = '_metadata'",
            [],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);
    if !exists {
        return Ok(0);
    }
    let v: Option<String> = conn
        .query_row(
            "SELECT value FROM _metadata WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    Ok(v.and_then(|s| s.parse().ok()).unwrap_or(0))
}

/// Apply all pending migrations to bring the DB to the latest version.
///
/// Each migration runs in its own transaction. `on_progress` is called
/// before and after each step so a UI can display progress.
pub fn migrate<F>(conn: &mut Connection, mut on_progress: F) -> Result<u32, MigrationsError>
where
    F: FnMut(MigrationProgress),
{
    let target = current_target();
    let current = schema_version(conn)?;

    if current > target {
        return Err(MigrationsError::FutureSchema {
            found: current,
            supported: target,
        });
    }
    if current == target {
        on_progress(MigrationProgress::AlreadyUpToDate { version: current });
        return Ok(current);
    }

    on_progress(MigrationProgress::Starting {
        from: current,
        to: target,
    });

    for (i, (name, sql)) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        let new_version = (i + 1) as u32;
        on_progress(MigrationProgress::Step {
            index: new_version,
            name,
        });
        let tx = conn.transaction()?;
        tx.execute_batch(sql)
            .map_err(|source| MigrationsError::StepFailed { name, source })?;
        tx.execute(
            "INSERT INTO _metadata (key, value) VALUES ('schema_version', ?1) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [new_version.to_string()],
        )?;
        tx.commit()?;
    }

    on_progress(MigrationProgress::Complete { version: target });
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn open_in_memory() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", true).unwrap();
        conn
    }

    #[test]
    fn schema_version_on_fresh_db_is_zero() {
        let conn = open_in_memory();
        assert_eq!(schema_version(&conn).unwrap(), 0);
    }

    #[test]
    fn migrate_applies_to_latest() {
        let mut conn = open_in_memory();
        let mut events = Vec::new();
        let v = migrate(&mut conn, |p| events.push(p)).unwrap();
        assert_eq!(v, current_target());
        assert_eq!(schema_version(&conn).unwrap(), current_target());
        assert!(events
            .iter()
            .any(|e| matches!(e, MigrationProgress::Starting { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, MigrationProgress::Complete { .. })));
    }

    #[test]
    fn migrate_is_idempotent() {
        let mut conn = open_in_memory();
        migrate(&mut conn, |_| {}).unwrap();
        // Run again — should be a no-op.
        let mut events = Vec::new();
        migrate(&mut conn, |p| events.push(p)).unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            MigrationProgress::AlreadyUpToDate { .. }
        ));
    }

    #[test]
    fn migrate_creates_expected_tables() {
        let mut conn = open_in_memory();
        migrate(&mut conn, |_| {}).unwrap();

        let expected = [
            "_metadata",
            "profile",
            "license",
            "competence",
            "progression",
            "session",
            "turn",
            "document",
            "evaluation",
            "event",
            "backup",
        ];
        for table in expected {
            let count: u32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "table `{table}` should exist after migrations");
        }
    }

    #[test]
    fn future_schema_is_rejected() {
        let mut conn = open_in_memory();
        // Manually set schema_version to something in the future.
        conn.execute_batch(
            "CREATE TABLE _metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL); \
             INSERT INTO _metadata (key, value) VALUES ('schema_version', '999');",
        )
        .unwrap();
        let err = migrate(&mut conn, |_| {}).unwrap_err();
        assert!(matches!(
            err,
            MigrationsError::FutureSchema { found: 999, .. }
        ));
    }

    #[test]
    fn open_with_migrations_works_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("andrea-test.db");
        let conn = crate::open_with_migrations(&path, |_| {}).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), current_target());
        // Reopen and verify idempotency.
        drop(conn);
        let conn2 = crate::open_with_migrations(&path, |_| {}).unwrap();
        assert_eq!(schema_version(&conn2).unwrap(), current_target());
    }

    #[test]
    fn profile_singleton_constraint_works() {
        let mut conn = open_in_memory();
        migrate(&mut conn, |_| {}).unwrap();
        conn.execute(
            "INSERT INTO profile (id, prenom, appel, email, contexte, niveau_depart, objectif, cadence) \
             VALUES (1, 'Marie', 'Marie', 'm@x.fr', 'CFA', 'debut', 'titre', 'libre')",
            [],
        )
        .unwrap();
        // Inserting a second profile with id != 1 should violate CHECK.
        let err = conn.execute(
            "INSERT INTO profile (id, prenom, appel, email, contexte, niveau_depart, objectif, cadence) \
             VALUES (2, 'Jean', 'Jean', 'j@x.fr', 'CFA', 'debut', 'titre', 'libre')",
            [],
        );
        assert!(err.is_err());
    }
}
