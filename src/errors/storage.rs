use super::{ascii_escape, ascii_path};
use core::fmt;
use std::ffi::OsStr;
use std::io;
use std::path::PathBuf;
use std::time::SystemTimeError;
#[expect(clippy::exhaustive_enums, reason = "closed storage diagnostics")]
#[expect(clippy::module_name_repetitions, reason = "clearer at call sites")]
#[derive(Debug)]
pub enum StorageError {
    Clock {
        source: SystemTimeError,
    },
    CorruptState {
        path: PathBuf,
        detail: String,
    },
    InvalidPath {
        action: &'static str,
        path: PathBuf,
    },
    InvalidSessionDirectory {
        name: String,
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
    TempCounterOverflow,
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
    pub fn invalid_session_directory(name: &OsStr, detail: String) -> Self {
        Self::InvalidSessionDirectory {
            name: ascii_escape(&name.to_string_lossy()),
            detail,
        }
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
}
impl fmt::Display for StorageError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[expect(clippy::pattern_type_mismatch, reason = "borrowed match avoids moves")]
        match self {
            Self::Clock { .. } => write!(f, "storage error: system clock is before epoch"),
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
            Self::InvalidSessionDirectory { name, detail } => write!(
                f,
                "storage error: invalid session directory '{name}': {detail}",
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
            Self::TempCounterOverflow => write!(f, "storage error: temp counter overflow"),
        }
    }
}
