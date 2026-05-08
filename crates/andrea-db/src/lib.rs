//! ANDREA local database (SQLite) with versioned migrations.
//!
//! Migrations are embedded at compile time via `include_str!`. The runner
//! is intentionally minimal — see [`migrate`] for the algorithm.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod migrations;

pub use migrations::{
    current_target, migrate, schema_version, MigrationProgress, MigrationsError, MIGRATIONS,
};

use rusqlite::Connection;

/// Open a SQLite connection at `path`, applying migrations as needed.
///
/// `on_progress` is invoked for each migration step so the UI can render a
/// progress bar. Pass `|_| {}` to ignore progress events.
pub fn open_with_migrations<P, F>(path: P, on_progress: F) -> Result<Connection, MigrationsError>
where
    P: AsRef<std::path::Path>,
    F: FnMut(MigrationProgress),
{
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", true)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    migrate(&mut conn, on_progress)?;
    Ok(conn)
}
