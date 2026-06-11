use crate::errors::{StorageError, ascii_path};
use core::sync::atomic::{AtomicU64, Ordering};
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);
pub(super) fn write(path: &Path, text: &str) -> Result<(), StorageError> {
    let parent = path.parent().ok_or_else(|| {
        StorageError::invalid_path("write file without parent", path.to_path_buf())
    })?;
    fs::create_dir_all(parent).map_err(|source| {
        StorageError::io("create parent directory", parent.to_path_buf(), source)
    })?;
    let temporary = temporary_path(path)?;
    fs::write(&temporary, text)
        .map_err(|source| StorageError::io("write temp file", temporary.clone(), source))?;
    match fs::rename(&temporary, path) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
            replace_existing_file(&temporary, path)
        }
        Err(source) => {
            if let Err(cleanup_error) = fs::remove_file(&temporary) {
                eprintln!(
                    "warning: failed to remove temp file '{}': {}",
                    ascii_path(&temporary),
                    cleanup_error,
                );
            }
            Err(StorageError::io("replace file", path.to_path_buf(), source))
        }
    }
}
fn replace_existing_file(temporary: &Path, path: &Path) -> Result<(), StorageError> {
    fs::remove_file(path)
        .map_err(|source| StorageError::io("remove old file", path.to_path_buf(), source))?;
    fs::rename(temporary, path)
        .map_err(|source| StorageError::io("replace file", path.to_path_buf(), source))
}
fn temporary_path(path: &Path) -> Result<PathBuf, StorageError> {
    let file_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| StorageError::invalid_path("build temp path", path.to_path_buf()))?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|source| StorageError::Clock { source })?
        .as_nanos();
    let counter = next_temp_id()?;
    let temporary_file_name = format!(
        "{file_name}.{}.{}.{}.tmp",
        process::id(),
        timestamp,
        counter
    );
    Ok(path.with_file_name(temporary_file_name))
}
fn next_temp_id() -> Result<u64, StorageError> {
    NEXT_TEMP_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .map_err(|_previous_value| StorageError::TempCounterOverflow)
}
