use crate::errors::{StorageError, ascii_path};
use core::time::Duration;
use rusqlite::{Connection, OpenFlags};
use std::path::Path;
const SCHEMA_VERSION: i64 = 1;
const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(30);
const WAL_JOURNAL_MODE: &str = "wal";
const CREATE_SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS moves (
    sequence INTEGER PRIMARY KEY CHECK(sequence >= 1),
    row INTEGER NOT NULL CHECK(row BETWEEN 0 AND 14),
    column INTEGER NOT NULL CHECK(column BETWEEN 0 AND 14),
    UNIQUE(row, column)
);
PRAGMA user_version = 1;
";
pub(super) fn open_writable(path: &Path) -> Result<Connection, StorageError> {
    let connection = Connection::open(path)
        .map_err(|source| StorageError::sqlite("open database", path.to_path_buf(), source))?;
    configure_connection(&connection, path)?;
    enable_write_ahead_logging(&connection, path)?;
    ensure_schema(&connection, path)?;
    Ok(connection)
}
pub(super) fn open_existing(path: &Path) -> Result<Option<Connection>, StorageError> {
    if !path_exists(path)? {
        return Ok(None);
    }
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|source| StorageError::sqlite("open database", path.to_path_buf(), source))?;
    configure_connection(&connection, path)?;
    validate_schema(&connection, path)?;
    Ok(Some(connection))
}
fn configure_connection(connection: &Connection, path: &Path) -> Result<(), StorageError> {
    connection
        .busy_timeout(SQLITE_BUSY_TIMEOUT)
        .map_err(|source| {
            StorageError::sqlite("configure busy timeout", path.to_path_buf(), source)
        })
}
fn enable_write_ahead_logging(connection: &Connection, path: &Path) -> Result<(), StorageError> {
    let journal_mode = connection
        .query_row("PRAGMA journal_mode = WAL", [], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|source| {
            StorageError::sqlite("enable write-ahead logging", path.to_path_buf(), source)
        })?;
    if journal_mode.eq_ignore_ascii_case(WAL_JOURNAL_MODE) {
        Ok(())
    } else {
        Err(corrupt_owned(
            path,
            format!("expected SQLite journal mode {WAL_JOURNAL_MODE}, found {journal_mode}"),
        ))
    }
}
fn ensure_schema(connection: &Connection, path: &Path) -> Result<(), StorageError> {
    let version = schema_version(connection, path)?;
    if version == SCHEMA_VERSION {
        return Ok(());
    }
    if version != 0 {
        return Err(unexpected_schema_version(path, version));
    }
    initialize_schema(connection, path)
}
fn initialize_schema(connection: &Connection, path: &Path) -> Result<(), StorageError> {
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(|source| {
            StorageError::sqlite("begin schema initialization", path.to_path_buf(), source)
        })?;
    let initialization = initialize_schema_transaction(connection, path);
    match initialization {
        Ok(()) => connection.execute_batch("COMMIT").map_err(|source| {
            StorageError::sqlite("commit schema initialization", path.to_path_buf(), source)
        }),
        Err(error) => {
            if let Err(rollback_error) = connection.execute_batch("ROLLBACK") {
                eprintln!(
                    "warning: failed to roll back schema initialization for '{}': {}",
                    ascii_path(path),
                    rollback_error,
                );
            }
            Err(error)
        }
    }
}
fn initialize_schema_transaction(connection: &Connection, path: &Path) -> Result<(), StorageError> {
    let version = schema_version(connection, path)?;
    if version == SCHEMA_VERSION {
        return Ok(());
    }
    if version != 0 {
        return Err(unexpected_schema_version(path, version));
    }
    if user_table_count(connection, path)? != 0 {
        return Err(corrupt_owned(
            path,
            "schema version is empty but database already contains tables".to_owned(),
        ));
    }
    connection
        .execute_batch(CREATE_SCHEMA_SQL)
        .map_err(|source| StorageError::sqlite("initialize schema", path.to_path_buf(), source))
}
fn validate_schema(connection: &Connection, path: &Path) -> Result<(), StorageError> {
    let version = schema_version(connection, path)?;
    if version == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(unexpected_schema_version(path, version))
    }
}
fn schema_version(connection: &Connection, path: &Path) -> Result<i64, StorageError> {
    connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|source| StorageError::sqlite("read schema version", path.to_path_buf(), source))
}
fn user_table_count(connection: &Connection, path: &Path) -> Result<usize, StorageError> {
    let count = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|source| {
            StorageError::sqlite("inspect schema tables", path.to_path_buf(), source)
        })?;
    usize::try_from(count).map_err(|error| {
        corrupt_owned(
            path,
            format!("schema table count contains invalid SQLite integer {count}: {error}"),
        )
    })
}
fn path_exists(path: &Path) -> Result<bool, StorageError> {
    path.try_exists()
        .map_err(|source| StorageError::io("check database existence", path.to_path_buf(), source))
}
fn unexpected_schema_version(path: &Path, version: i64) -> StorageError {
    corrupt_owned(
        path,
        format!("expected schema version {SCHEMA_VERSION}, found {version}"),
    )
}
fn corrupt_owned(path: &Path, detail: String) -> StorageError {
    StorageError::corrupt_state(path.to_path_buf(), detail)
}
