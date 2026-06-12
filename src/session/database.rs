use super::{MoveSnapshot, WaitSnapshot};
use crate::board::Board;
use crate::coordinate::Coordinate;
use crate::errors::{IllegalMove, StorageError};
use rusqlite::TransactionBehavior;
use std::path::Path;
mod integer;
mod records;
mod schema;
mod summary;
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum StoredSubmission {
    Illegal(IllegalMove),
    Legal { sequence: usize },
    Won { sequence: usize },
}
pub(super) fn submit(
    path: &Path,
    coordinate: Coordinate,
) -> Result<StoredSubmission, StorageError> {
    let mut connection = schema::open_writable(path)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|source| {
            StorageError::sqlite("begin write transaction", path.to_path_buf(), source)
        })?;
    let mut board = records::read_board_from(&transaction, path)?;
    match board.place(coordinate) {
        Ok(placed) => {
            records::insert_move(&transaction, path, placed.sequence, coordinate)?;
            transaction.commit().map_err(|source| {
                StorageError::sqlite("commit write transaction", path.to_path_buf(), source)
            })?;
            if placed.won {
                Ok(StoredSubmission::Won {
                    sequence: placed.sequence,
                })
            } else {
                Ok(StoredSubmission::Legal {
                    sequence: placed.sequence,
                })
            }
        }
        Err(error) => Ok(StoredSubmission::Illegal(error)),
    }
}
pub(super) fn read_board(path: &Path) -> Result<Option<Board>, StorageError> {
    let Some(connection) = schema::open_existing(path)? else {
        return Ok(None);
    };
    records::read_board_from(&connection, path).map(Some)
}
pub(super) fn read_move_count(path: &Path) -> Result<Option<usize>, StorageError> {
    let Some(connection) = schema::open_existing(path)? else {
        return Ok(None);
    };
    summary::move_count(&connection, path).map(Some)
}
pub(super) fn read_move_snapshot(
    snapshot: &WaitSnapshot,
) -> Result<Option<MoveSnapshot>, StorageError> {
    let path = snapshot.database_file();
    let Some(connection) = schema::open_existing(path)? else {
        return Ok(None);
    };
    let sequence = snapshot.sequence();
    let move_records = records::read_records_through(&connection, path, sequence)?;
    if move_records.len() < sequence {
        return wait_for_missing_sequence(&connection, path, sequence, move_records.len());
    }
    let board = records::replay_records(path, &move_records)?;
    let Some(last_record) = move_records.last() else {
        return Err(corrupt(path, "snapshot sequence must be at least one"));
    };
    let lost = board.winner().is_some();
    Ok(Some(MoveSnapshot {
        board,
        coordinate: last_record.coordinate,
        lost,
    }))
}
fn wait_for_missing_sequence(
    connection: &rusqlite::Connection,
    path: &Path,
    sequence: usize,
    records: usize,
) -> Result<Option<MoveSnapshot>, StorageError> {
    if records::max_sequence(connection, path)? >= sequence {
        Err(corrupt_owned(
            path,
            format!("snapshot sequence {sequence} is missing after {records} records"),
        ))
    } else {
        Ok(None)
    }
}
fn corrupt(path: &Path, detail: &'static str) -> StorageError {
    StorageError::corrupt_state(path.to_path_buf(), detail.to_owned())
}
fn corrupt_owned(path: &Path, detail: String) -> StorageError {
    StorageError::corrupt_state(path.to_path_buf(), detail)
}
