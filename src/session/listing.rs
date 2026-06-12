use super::{SessionSummary, database};
use crate::errors::StorageError;
use crate::session_id::SessionId;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
const SQLITE_EXTENSION: &str = "sqlite";
#[derive(Clone, Debug, Eq, PartialEq)]
struct SessionDatabase {
    id: SessionId,
    path: PathBuf,
}
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
    collect_entries(root, entries)
}
fn collect_entries(root: &Path, entries: fs::ReadDir) -> Result<Vec<SessionSummary>, StorageError> {
    let mut databases = Vec::new();
    for entry_result in entries {
        let entry = entry_result
            .map_err(|source| StorageError::io("read session entry", PathBuf::new(), source))?;
        let file_name = entry.file_name();
        let file_name_path = Path::new(&file_name);
        let file_type = entry.file_type().map_err(|source| {
            StorageError::io("read session entry type", root.join(&file_name), source)
        })?;
        if file_type.is_dir() || !has_sqlite_extension(file_name_path) {
            continue;
        }
        let path = root.join(&file_name);
        let session_id = session_id_from_database_file(&path)?;
        databases.push(SessionDatabase {
            id: session_id,
            path,
        });
    }
    databases.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    summarize_databases(root, &databases)
}
fn summarize_databases(
    root: &Path,
    databases: &[SessionDatabase],
) -> Result<Vec<SessionSummary>, StorageError> {
    if databases.len() <= 1 {
        return summarize_database_slice(databases);
    }
    let workers = worker_count(root, databases.len())?;
    let chunk_size = databases.len().div_ceil(workers);
    thread::scope(|scope| {
        let handles = databases
            .chunks(chunk_size)
            .map(|chunk| scope.spawn(move || summarize_database_slice(chunk)))
            .collect::<Vec<_>>();
        let mut summaries = Vec::with_capacity(databases.len());
        for handle in handles {
            let chunk_result = match handle.join() {
                Ok(value) => value,
                Err(payload) => std::panic::resume_unwind(payload),
            };
            let mut chunk_summaries = chunk_result?;
            summaries.append(&mut chunk_summaries);
        }
        Ok(summaries)
    })
}
fn summarize_database_slice(
    databases: &[SessionDatabase],
) -> Result<Vec<SessionSummary>, StorageError> {
    let mut summaries = Vec::with_capacity(databases.len());
    for database_file in databases {
        let Some(moves) = database::read_move_count(&database_file.path)? else {
            continue;
        };
        summaries.push(SessionSummary {
            id: database_file.id.clone(),
            moves,
        });
    }
    Ok(summaries)
}
fn worker_count(root: &Path, item_count: usize) -> Result<usize, StorageError> {
    let parallelism = thread::available_parallelism().map_err(|source| {
        StorageError::io("detect list parallelism", root.to_path_buf(), source)
    })?;
    Ok(item_count.min(parallelism.get()))
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
