use crate::coordinate::Coordinate;
use core::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTimeError;
mod formatting;
#[expect(clippy::exhaustive_enums, reason = "closed CLI error domains")]
#[derive(Debug)]
pub enum AppError {
    Input(InputError),
    Storage(StorageError),
}
impl fmt::Display for AppError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[expect(clippy::pattern_type_mismatch, reason = "borrowed match avoids moves")]
        match self {
            Self::Input(error) => write!(f, "invalid input: {error}"),
            Self::Storage(error) => write!(f, "{error}"),
        }
    }
}
impl From<InputError> for AppError {
    #[inline]
    fn from(value: InputError) -> Self {
        Self::Input(value)
    }
}
impl From<StorageError> for AppError {
    #[inline]
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}
#[expect(clippy::exhaustive_enums, reason = "closed input validation failures")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InputError {
    ExpectedFields {
        found: usize,
    },
    EmptyRoom,
    RoomTooLong {
        max: usize,
    },
    NonAsciiRoom,
    Coordinate {
        axis: &'static str,
        value: String,
        reason: &'static str,
    },
}
impl fmt::Display for InputError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[expect(clippy::pattern_type_mismatch, reason = "borrowed match avoids moves")]
        match self {
            Self::ExpectedFields { found } => {
                write!(f, "expected three fields: room row column, found {found}")
            }
            Self::EmptyRoom => write!(f, "room id must not be empty"),
            Self::RoomTooLong { max } => {
                write!(
                    f,
                    "room id must contain at most {max} printable ASCII bytes"
                )
            }
            Self::NonAsciiRoom => {
                write!(f, "room id must contain printable ASCII bytes only")
            }
            Self::Coordinate {
                axis,
                value,
                reason,
            } => write!(
                f,
                "{axis} must be valid: {reason}; got '{}'",
                ascii_escape(value),
            ),
        }
    }
}
#[expect(clippy::exhaustive_enums, reason = "complete illegal move outcomes")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IllegalMove {
    BoardFull,
    InvalidCoordinate,
    Occupied { coordinate: Coordinate },
}
impl fmt::Display for IllegalMove {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[expect(clippy::pattern_type_mismatch, reason = "borrowed match avoids moves")]
        match self {
            Self::BoardFull => write!(f, "board is full"),
            Self::InvalidCoordinate => write!(f, "position is outside the board"),
            Self::Occupied { coordinate } => write!(
                f,
                "position {},{} is already occupied",
                coordinate.row_label(),
                coordinate.column_label(),
            ),
        }
    }
}
#[expect(clippy::exhaustive_enums, reason = "closed storage diagnostics")]
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
#[must_use]
#[expect(
    clippy::missing_inline_in_public_items,
    reason = "allocating escape helper"
)]
pub fn ascii_escape(value: &str) -> String {
    formatting::escape(value)
}
#[must_use]
#[inline]
pub fn ascii_path(path: &Path) -> String {
    formatting::path(path)
}
