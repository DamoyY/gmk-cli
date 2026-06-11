use crate::board::Board;
use crate::coordinate::{BOARD_CELLS, BOARD_SIZE, Coordinate};
use crate::errors::StorageError;
use crate::stone::Stone;
use std::path::Path;
const HEADER: &str = "gmk-cli session v1";
#[must_use]
#[expect(
    clippy::missing_inline_in_public_items,
    reason = "board encoding builds an owned persistence payload"
)]
pub fn encode_board(board: &Board) -> String {
    let mut output = String::new();
    output.push_str(HEADER);
    output.push('\n');
    output.push_str("moves ");
    output.push_str(&board.moves().to_string());
    output.push('\n');
    output.push_str("board\n");
    for row in 0..BOARD_SIZE {
        for column in 0..BOARD_SIZE {
            let coordinate = match Coordinate::new(row, column) {
                Ok(value) => value,
                Err(error) => panic!("internal coordinate generation failed: {error}"),
            };
            output.push(board.get(coordinate).map_or('*', Stone::board_char));
        }
        output.push('\n');
    }
    output
}
#[expect(
    clippy::missing_inline_in_public_items,
    reason = "board decoding is validation-heavy persistence logic"
)]
pub fn decode_board(path: &Path, text: &str) -> Result<Board, StorageError> {
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or_else(|| corrupt(path, "missing header"))?;
    if header != HEADER {
        return Err(corrupt(path, "invalid header"));
    }
    let moves_line = lines
        .next()
        .ok_or_else(|| corrupt(path, "missing move count"))?;
    let moves_text = moves_line
        .strip_prefix("moves ")
        .ok_or_else(|| corrupt(path, "invalid move count line"))?;
    let moves = match moves_text.parse::<usize>() {
        Ok(value) => value,
        Err(error) => {
            return Err(corrupt_owned(
                path,
                format!("move count is not a number: {error}"),
            ));
        }
    };
    let board_marker = lines
        .next()
        .ok_or_else(|| corrupt(path, "missing board marker"))?;
    if board_marker != "board" {
        return Err(corrupt(path, "invalid board marker"));
    }
    let mut cells = [None; BOARD_CELLS];
    for row in 0..BOARD_SIZE {
        let line = lines
            .next()
            .ok_or_else(|| corrupt(path, "missing board row"))?;
        if line.len() != BOARD_SIZE {
            return Err(corrupt(path, "board row has invalid length"));
        }
        for (column, byte) in line.bytes().enumerate() {
            let index = row * BOARD_SIZE + column;
            let Some(cell) = cells.get_mut(index) else {
                return Err(corrupt(path, "board cell index is outside the board"));
            };
            *cell = Stone::from_board_byte(byte).map_err(|detail| corrupt(path, detail))?;
        }
    }
    if lines.next().is_some() {
        return Err(corrupt(path, "unexpected trailing data"));
    }
    Board::from_parts(cells, moves).map_err(|detail| corrupt_owned(path, detail))
}
fn corrupt(path: &Path, detail: &'static str) -> StorageError {
    StorageError::corrupt_state(path.to_path_buf(), detail.to_owned())
}
fn corrupt_owned(path: &Path, detail: String) -> StorageError {
    StorageError::corrupt_state(path.to_path_buf(), detail)
}
