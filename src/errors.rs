use crate::coordinate::Coordinate;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;
#[expect(clippy::exhaustive_enums, reason = "closed CLI error domains")]
#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid input: {0}")]
    Input(#[from] InputError),
    #[error(transparent)]
    Storage(#[from] StorageError),
}
#[expect(clippy::exhaustive_enums, reason = "complete illegal move outcomes")]
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum IllegalMove {
    #[error("board is full")]
    BoardFull,
    #[error("game is already over")]
    GameOver,
    #[error("position is outside the board")]
    InvalidCoordinate,
    # [error ("position {},{} is already occupied" , . coordinate . row_label () , . coordinate . column_label ())]
    Occupied { coordinate: Coordinate },
}
#[expect(clippy::exhaustive_enums, reason = "closed input validation failures")]
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum InputError {
    #[error("session id must not be empty")]
    EmptySession,
    #[error("session argument is required")]
    MissingSessionArgument,
    #[error("session argument was provided twice")]
    DuplicateSessionArgument,
    #[error("session id must contain at most {max} printable ASCII bytes")]
    SessionTooLong { max: usize },
    #[error("session id must contain printable ASCII bytes only")]
    NonAsciiSession,
    #[error(
        "session id must be usable as a SQLite file name and must not contain '<', '>', ':', '\"', '/', '\\', '|', '?', or '*'"
    )]
    UnsafeSessionFileName,
    #[error("session id must not be a reserved Windows device name")]
    ReservedSessionFileName,
    # [error ("session '{}' does not exist" , ascii_escape (. session))]
    UnknownSession { session: String },
    # [error ("{axis} must be valid: {reason}; got '{}'" , ascii_escape (. value))]
    Coordinate {
        axis: &'static str,
        value: String,
        reason: &'static str,
    },
}
#[expect(clippy::exhaustive_enums, reason = "closed storage diagnostics")]
#[derive(Debug, Error)]
pub enum StorageError {
    # [error ("storage error: corrupt state at '{}': {detail}" , ascii_path (. path))]
    CorruptState { path: PathBuf, detail: String },
    # [error ("storage error: cannot {action} at '{}'" , ascii_path (. path))]
    InvalidPath { action: &'static str, path: PathBuf },
    # [error ("storage error: invalid session database '{}': {detail}" , ascii_path (. path))]
    InvalidSessionDatabase { path: PathBuf, detail: String },
    # [error ("storage error: failed to {action} at '{}': {}" , ascii_path (. path) , ascii_escape (&. source . to_string ()))]
    Io {
        action: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    # [error ("storage error: executable has no parent directory: '{}'" , ascii_path (. path))]
    NoExecutableDirectory { path: PathBuf },
    # [error ("storage error: failed to {action} while watching '{}': {}" , ascii_path (. path) , ascii_escape (&. source . to_string ()))]
    Notify {
        action: &'static str,
        path: PathBuf,
        source: notify::Error,
    },
    # [error ("storage error: failed to {action} in SQLite database '{}': {}" , ascii_path (. path) , ascii_escape (&. source . to_string ()))]
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
    pub const fn notify(action: &'static str, path: PathBuf, source: notify::Error) -> Self {
        Self::Notify {
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
#[must_use]
#[expect(
    clippy::missing_inline_in_public_items,
    reason = "allocating escape helper"
)]
pub fn ascii_escape(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        for escaped in core::ascii::escape_default(byte) {
            output.push(char::from(escaped));
        }
    }
    output
}
#[must_use]
#[inline]
pub fn ascii_path(path: &Path) -> String {
    ascii_escape(&path.to_string_lossy())
}
