use crate::board::Board;
use crate::coordinate::Coordinate;
use crate::errors::StorageError;
use crate::persistence::{decode_board, encode_board};
use std::path::Path;
const HEADER: &str = "gmk-cli snapshot v1";
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MoveSnapshot {
    pub(crate) board: Board,
    pub(crate) coordinate: Coordinate,
    pub(crate) lost: bool,
}
pub(crate) fn encode(snapshot: &MoveSnapshot) -> String {
    let mut output = String::new();
    output.push_str(HEADER);
    output.push('\n');
    output.push_str("row ");
    output.push_str(&snapshot.coordinate.row_label());
    output.push('\n');
    output.push_str("column ");
    output.push(snapshot.coordinate.column_label());
    output.push('\n');
    output.push_str("lost ");
    output.push_str(if snapshot.lost { "true" } else { "false" });
    output.push('\n');
    output.push_str(&encode_board(&snapshot.board));
    output
}
pub(crate) fn decode(path: &Path, text: &str) -> Result<MoveSnapshot, StorageError> {
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or_else(|| corrupt(path, "missing snapshot header"))?;
    if header != HEADER {
        return Err(corrupt(path, "invalid snapshot header"));
    }
    let row = parse_prefixed_line(path, lines.next(), "row ", "row")?;
    let column = parse_prefixed_line(path, lines.next(), "column ", "column")?;
    let lost_text = parse_prefixed_line(path, lines.next(), "lost ", "lost")?;
    let lost = parse_lost(path, lost_text)?;
    let coordinate = Coordinate::parse(row, column)
        .map_err(|error| corrupt_owned(path, format!("invalid snapshot coordinate: {error}")))?;
    let board_text = collect_remaining_lines(lines);
    let board = decode_board(path, &board_text)?;
    Ok(MoveSnapshot {
        board,
        coordinate,
        lost,
    })
}
fn parse_prefixed_line<'line>(
    path: &Path,
    line: Option<&'line str>,
    prefix: &'static str,
    name: &'static str,
) -> Result<&'line str, StorageError> {
    let line_value = line.ok_or_else(|| corrupt_owned(path, format!("missing {name} line")))?;
    line_value
        .strip_prefix(prefix)
        .ok_or_else(|| corrupt_owned(path, format!("invalid {name} line")))
}
fn parse_lost(path: &Path, value: &str) -> Result<bool, StorageError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(corrupt(path, "invalid lost value")),
    }
}
fn collect_remaining_lines<'line>(lines: impl Iterator<Item = &'line str>) -> String {
    let mut output = String::new();
    for line in lines {
        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str(line);
    }
    output
}
fn corrupt(path: &Path, detail: &'static str) -> StorageError {
    StorageError::corrupt_state(path.to_path_buf(), detail.to_owned())
}
fn corrupt_owned(path: &Path, detail: String) -> StorageError {
    StorageError::corrupt_state(path.to_path_buf(), detail)
}
