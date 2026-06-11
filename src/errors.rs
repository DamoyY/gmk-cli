use crate::coordinate::Coordinate;
use core::fmt;
use std::path::Path;
mod formatting;
pub mod input;
pub mod storage;
pub type InputError = input::InputError;
pub type StorageError = storage::StorageError;
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
#[expect(clippy::exhaustive_enums, reason = "complete illegal move outcomes")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IllegalMove {
    BoardFull,
    GameOver,
    InvalidCoordinate,
    Occupied { coordinate: Coordinate },
}
impl fmt::Display for IllegalMove {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[expect(clippy::pattern_type_mismatch, reason = "borrowed match avoids moves")]
        match self {
            Self::BoardFull => write!(f, "board is full"),
            Self::GameOver => write!(f, "game is already over"),
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
