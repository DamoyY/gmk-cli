use super::integer::sql_to_usize;
use crate::coordinate::BOARD_CELLS;
use crate::errors::StorageError;
use rusqlite::Connection;
use std::path::Path;
pub(super) fn move_count(connection: &Connection, path: &Path) -> Result<usize, StorageError> {
    let count = count_rows(connection, path)?;
    if count == 0 {
        return Ok(0);
    }
    let minimum_sequence = sequence_boundary(
        connection,
        path,
        "read first move sequence",
        "SELECT sequence FROM moves ORDER BY sequence LIMIT 1",
    )?;
    let maximum_sequence = sequence_boundary(
        connection,
        path,
        "read last move sequence",
        "SELECT sequence FROM moves ORDER BY sequence DESC LIMIT 1",
    )?;
    validate_move_count(path, count, minimum_sequence, maximum_sequence)?;
    Ok(count)
}
fn count_rows(connection: &Connection, path: &Path) -> Result<usize, StorageError> {
    let count_sql = connection
        .query_row("SELECT COUNT(*) FROM moves", [], |row| row.get::<_, i64>(0))
        .map_err(|source| {
            StorageError::sqlite("read summarized move count", path.to_path_buf(), source)
        })?;
    sql_to_usize(path, "move count", count_sql)
}
fn sequence_boundary(
    connection: &Connection,
    path: &Path,
    action: &'static str,
    query: &'static str,
) -> Result<usize, StorageError> {
    let sequence_sql = connection
        .query_row(query, [], |row| row.get::<_, i64>(0))
        .map_err(|source| StorageError::sqlite(action, path.to_path_buf(), source))?;
    sql_to_usize(path, "sequence", sequence_sql)
}
fn validate_move_count(
    path: &Path,
    count: usize,
    minimum_sequence: usize,
    maximum_sequence: usize,
) -> Result<(), StorageError> {
    if count > BOARD_CELLS {
        return Err(corrupt_owned(
            path,
            format!("move count {count} exceeds board capacity {BOARD_CELLS}"),
        ));
    }
    if minimum_sequence != 1 || maximum_sequence != count {
        return Err(corrupt_owned(
            path,
            format!(
                "move sequence is not contiguous: count {count}, minimum {minimum_sequence}, maximum {maximum_sequence}",
            ),
        ));
    }
    Ok(())
}
fn corrupt_owned(path: &Path, detail: String) -> StorageError {
    StorageError::corrupt_state(path.to_path_buf(), detail)
}
