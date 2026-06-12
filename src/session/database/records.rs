use crate::board::Board;
use crate::coordinate::Coordinate;
use crate::errors::StorageError;
use rusqlite::{Connection, params};
use std::path::Path;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct MoveRecord {
    pub(super) sequence: usize,
    pub(super) coordinate: Coordinate,
}
pub(super) fn read_board_from(connection: &Connection, path: &Path) -> Result<Board, StorageError> {
    let records = read_all_records(connection, path)?;
    replay_records(path, &records)
}
pub(super) fn read_records_through(
    connection: &Connection,
    path: &Path,
    sequence: usize,
) -> Result<Vec<MoveRecord>, StorageError> {
    let sequence_sql = usize_to_sql(path, "sequence", sequence)?;
    let mut statement = connection
        .prepare("SELECT sequence, row, column FROM moves WHERE sequence <= ?1 ORDER BY sequence")
        .map_err(|source| {
            StorageError::sqlite("prepare snapshot query", path.to_path_buf(), source)
        })?;
    let rows = statement
        .query_map(params![sequence_sql], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|source| {
            StorageError::sqlite("query snapshot moves", path.to_path_buf(), source)
        })?;
    collect_records(path, rows)
}
pub(super) fn replay_records(path: &Path, records: &[MoveRecord]) -> Result<Board, StorageError> {
    let mut board = Board::empty();
    for (index, record) in records.iter().enumerate() {
        let expected_sequence = index + 1;
        if record.sequence != expected_sequence {
            return Err(corrupt_owned(
                path,
                format!(
                    "move sequence is not contiguous: expected {expected_sequence}, found {}",
                    record.sequence,
                ),
            ));
        }
        board
            .place(record.coordinate)
            .map_err(|error| corrupt_owned(path, format!("stored move is illegal: {error}")))?;
    }
    Ok(board)
}
pub(super) fn insert_move(
    connection: &Connection,
    path: &Path,
    sequence: usize,
    coordinate: Coordinate,
) -> Result<(), StorageError> {
    let sequence_sql = usize_to_sql(path, "sequence", sequence)?;
    let row_sql = usize_to_sql(path, "row", coordinate.row())?;
    let column_sql = usize_to_sql(path, "column", coordinate.column())?;
    connection
        .execute(
            "INSERT INTO moves(sequence, row, column) VALUES (?1, ?2, ?3)",
            params![sequence_sql, row_sql, column_sql],
        )
        .map(|_rows| ())
        .map_err(|source| StorageError::sqlite("insert move", path.to_path_buf(), source))
}
pub(super) fn max_sequence(connection: &Connection, path: &Path) -> Result<usize, StorageError> {
    let max_value = connection
        .query_row("SELECT MAX(sequence) FROM moves", [], |row| {
            row.get::<_, Option<i64>>(0)
        })
        .map_err(|source| {
            StorageError::sqlite("read maximum sequence", path.to_path_buf(), source)
        })?;
    max_value.map_or_else(|| Ok(0), |value| sql_to_usize(path, "sequence", value))
}
fn read_all_records(connection: &Connection, path: &Path) -> Result<Vec<MoveRecord>, StorageError> {
    let mut statement = connection
        .prepare("SELECT sequence, row, column FROM moves ORDER BY sequence")
        .map_err(|source| StorageError::sqlite("prepare move query", path.to_path_buf(), source))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|source| StorageError::sqlite("query moves", path.to_path_buf(), source))?;
    collect_records(path, rows)
}
fn collect_records(
    path: &Path,
    rows: impl Iterator<Item = rusqlite::Result<(i64, i64, i64)>>,
) -> Result<Vec<MoveRecord>, StorageError> {
    let mut records = Vec::new();
    for row in rows {
        let (sequence, row_index, column_index) = row
            .map_err(|source| StorageError::sqlite("read move row", path.to_path_buf(), source))?;
        records.push(parse_record(path, sequence, row_index, column_index)?);
    }
    Ok(records)
}
fn parse_record(
    path: &Path,
    sequence: i64,
    row_index: i64,
    column_index: i64,
) -> Result<MoveRecord, StorageError> {
    let sequence_value = sql_to_usize(path, "sequence", sequence)?;
    let row_value = sql_to_usize(path, "row", row_index)?;
    let column_value = sql_to_usize(path, "column", column_index)?;
    let coordinate = Coordinate::new(row_value, column_value)
        .map_err(|error| corrupt_owned(path, format!("invalid coordinate in database: {error}")))?;
    Ok(MoveRecord {
        sequence: sequence_value,
        coordinate,
    })
}
fn usize_to_sql(path: &Path, field: &'static str, value: usize) -> Result<i64, StorageError> {
    i64::try_from(value).map_err(|error| {
        corrupt_owned(
            path,
            format!("{field} does not fit in SQLite integer: {error}"),
        )
    })
}
fn sql_to_usize(path: &Path, field: &'static str, value: i64) -> Result<usize, StorageError> {
    usize::try_from(value).map_err(|error| {
        corrupt_owned(
            path,
            format!("{field} contains invalid SQLite integer {value}: {error}"),
        )
    })
}
fn corrupt_owned(path: &Path, detail: String) -> StorageError {
    StorageError::corrupt_state(path.to_path_buf(), detail)
}
