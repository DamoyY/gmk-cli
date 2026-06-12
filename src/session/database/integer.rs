use crate::errors::StorageError;
use std::path::Path;
pub(super) fn usize_to_sql(
    path: &Path,
    field: &'static str,
    value: usize,
) -> Result<i64, StorageError> {
    i64::try_from(value).map_err(|error| {
        corrupt_owned(
            path,
            format!("{field} does not fit in SQLite integer: {error}"),
        )
    })
}
pub(super) fn sql_to_usize(
    path: &Path,
    field: &'static str,
    value: i64,
) -> Result<usize, StorageError> {
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
