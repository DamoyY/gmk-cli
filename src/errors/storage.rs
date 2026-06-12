use super::{ascii_escape, ascii_path};
use core::fmt;
use std::io;
use std::path::PathBuf;
#[expect(clippy::exhaustive_enums, reason = "closed storage diagnostics")]
#[expect(clippy::module_name_repetitions, reason = "clearer at call sites")]
#[derive(Debug)]
pub enum StorageError {
    CorruptState {
        path: PathBuf,
        detail: String,
    },
    InvalidPath {
        action: &'static str,
        path: PathBuf,
    },
    InvalidSessionDatabase {
        path: PathBuf,
        detail: String,
    },
    Io {
        action: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    NoExecutableDirectory {
        path: PathBuf,
    },
    Sqlite {
        action: &'static str,
        path: PathBuf,
        source: rusqlite::Error,
    },
}
impl StorageError {
    #[must_use]
    #[inline]
    pub const fn corrupt_state(path: PathBuf, detail: String) -> Self {
        Self::CorruptState { path, detail }
    }
    #[must_use]
    #[inline]
    pub const fn invalid_path(action: &'static str, path: PathBuf) -> Self {
        Self::InvalidPath { action, path }
    }
    #[must_use]
    #[inline]
    pub const fn invalid_session_database(path: PathBuf, detail: String) -> Self {
        Self::InvalidSessionDatabase { path, detail }
    }
    #[must_use]
    #[inline]
    pub const fn io(action: &'static str, path: PathBuf, source: io::Error) -> Self {
        Self::Io {
            action,
            path,
            source,
        }
    }
    #[must_use]
    #[inline]
    pub const fn sqlite(action: &'static str, path: PathBuf, source: rusqlite::Error) -> Self {
        Self::Sqlite {
            action,
            path,
            source,
        }
    }
}
impl fmt::Display for StorageError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[expect(clippy::pattern_type_mismatch, reason = "borrowed match avoids moves")]
        match self {
            Self::CorruptState { path, detail } => write!(
                f,
                "storage error: corrupt state at '{}': {detail}",
                ascii_path(path),
            ),
            Self::InvalidPath { action, path } => {
                write!(
                    f,
                    "storage error: cannot {action} at '{}'",
                    ascii_path(path)
                )
            }
            Self::InvalidSessionDatabase { path, detail } => write!(
                f,
                "storage error: invalid session database '{}': {detail}",
                ascii_path(path),
            ),
            Self::Io {
                action,
                path,
                source,
            } => write!(
                f,
                "storage error: failed to {action} at '{}': {}",
                ascii_path(path),
                ascii_escape(&source.to_string()),
            ),
            Self::NoExecutableDirectory { path } => write!(
                f,
                "storage error: executable has no parent directory: '{}'",
                ascii_path(path),
            ),
            Self::Sqlite {
                action,
                path,
                source,
            } => write!(
                f,
                "storage error: failed to {action} in SQLite database '{}': {}",
                ascii_path(path),
                ascii_escape(&source.to_string()),
            ),
        }
    }
}
