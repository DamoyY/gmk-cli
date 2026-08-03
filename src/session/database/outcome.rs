use super::integer::{sql_to_usize, usize_to_sql};
use crate::errors::StorageError;
use rusqlite::OptionalExtension as _;
use rusqlite::{Connection, params};
use std::path::Path;
pub(super) fn resigned_after_sequence(
    connection: &Connection,
    path: &Path,
) -> Result<Option<usize>, StorageError> {
    let value = connection
        .query_row(
            "SELECT after_sequence FROM resignation WHERE singleton = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|source| StorageError::sqlite("read resignation", path.to_path_buf(), source))?;
    value
        .map(|sequence| sql_to_usize(path, "resignation sequence", sequence))
        .transpose()
}
pub(super) fn insert_resignation(
    connection: &Connection,
    path: &Path,
    after_sequence: usize,
) -> Result<(), StorageError> {
    let sequence_sql = usize_to_sql(path, "resignation sequence", after_sequence)?;
    connection
        .execute(
            "INSERT INTO resignation(singleton, after_sequence) VALUES (1, ?1)",
            params![sequence_sql],
        )
        .map(|_rows| ())
        .map_err(|source| StorageError::sqlite("insert resignation", path.to_path_buf(), source))
}
