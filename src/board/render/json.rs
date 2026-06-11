use super::Board;
use crate::coordinate::{BOARD_SIZE, Coordinate};
use crate::stone::Stone;
impl Board {
    #[must_use]
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "board rendering allocates a complete JSON payload"
    )]
    pub fn render_json(&self) -> String {
        let mut output = String::new();
        output.push('[');
        for row in 0..BOARD_SIZE {
            if row > 0 {
                output.push(',');
            }
            push_json_row(self, &mut output, row);
        }
        output.push_str("]\n");
        output
    }
}
fn push_json_row(board: &Board, output: &mut String, row: usize) {
    output.push('[');
    for column in 0..BOARD_SIZE {
        if column > 0 {
            output.push(',');
        }
        output.push('"');
        output.push(cell_char(board, row, column));
        output.push('"');
    }
    output.push(']');
}
fn cell_char(board: &Board, row: usize, column: usize) -> char {
    let coordinate = match Coordinate::new(row, column) {
        Ok(value) => value,
        Err(error) => panic!("internal coordinate generation failed: {error}"),
    };
    board.get(coordinate).map_or('*', Stone::board_char)
}
