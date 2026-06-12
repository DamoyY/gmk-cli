use super::{SessionSummary, database};
use crate::errors::StorageError;
use crate::session_id::SessionId;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
const SQLITE_EXTENSION: &str = "sqlite";
pub(super) fn collect(root: &Path) -> Result<Vec<SessionSummary>, StorageError> {
    let entries = match fs::read_dir(root) {
        Ok(value) => value,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(StorageError::io(
                "list session directory",
                root.to_path_buf(),
                error,
            ));
        }
    };
    collect_entries(entries)
}
fn collect_entries(entries: fs::ReadDir) -> Result<Vec<SessionSummary>, StorageError> {
    let mut summaries = Vec::new();
    for entry_result in entries {
        let entry = entry_result
            .map_err(|source| StorageError::io("read session entry", PathBuf::new(), source))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| StorageError::io("read session entry type", path.clone(), source))?;
        if file_type.is_dir() || !has_sqlite_extension(&path) {
            continue;
        }
        let session_id = session_id_from_database_file(&path)?;
        let Some(board) = database::read_board(&path)? else {
            continue;
        };
        summaries.push(SessionSummary {
            id: session_id,
            moves: board.moves(),
        });
    }
    summaries.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    Ok(summaries)
}
fn has_sqlite_extension(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extension == SQLITE_EXTENSION)
}
fn session_id_from_database_file(path: &Path) -> Result<SessionId, StorageError> {
    let Some(stem) = path.file_stem().and_then(OsStr::to_str) else {
        return Err(StorageError::invalid_session_database(
            path.to_path_buf(),
            "database file name is not printable Unicode".to_owned(),
        ));
    };
    SessionId::parse(stem).map_err(|error| {
        StorageError::invalid_session_database(path.to_path_buf(), error.to_string())
    })
}
