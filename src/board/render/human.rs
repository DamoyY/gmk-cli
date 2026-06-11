use super::Board;
use crate::coordinate::{BOARD_SIZE, Coordinate, axis_label, number_label};
use crate::stone::Stone;
const RESET: &str = "\x1b[0m";
const AXIS_STYLE: &str = "\x1b[1;38;5;220m";
const GRID_STYLE: &str = "\x1b[38;5;65m";
const META_LABEL_STYLE: &str = "\x1b[1;38;5;81m";
const META_VALUE_STYLE: &str = "\x1b[1;38;5;220m";
const BLACK_STYLE: &str = "\x1b[1;38;5;250m";
const WHITE_STYLE: &str = "\x1b[1;38;5;231m";
const EMPTY_STYLE: &str = "\x1b[38;5;236m";
const CELL_BORDER: &str = "───";
const EMPTY_CELL: &str = "   ";
impl Board {
    #[must_use]
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "board rendering is terminal formatting"
    )]
    pub fn render_human(&self) -> String {
        let mut output = String::new();
        push_to_move(self, &mut output);
        push_board(self, &mut output);
        output
    }
    #[must_use]
    pub(crate) fn render_human_move(&self, coordinate: Coordinate) -> String {
        let mut output = String::new();
        push_diff(&mut output, coordinate);
        push_colored(&mut output, GRID_STYLE, "---");
        output.push('\n');
        push_to_move(self, &mut output);
        push_board(self, &mut output);
        output
    }
}
fn push_diff(output: &mut String, coordinate: Coordinate) {
    push_colored(output, META_LABEL_STYLE, "Diff:");
    output.push('\n');
    push_colored(output, META_LABEL_STYLE, "- row:");
    output.push(' ');
    push_colored(output, META_VALUE_STYLE, &coordinate.row_label());
    output.push('\n');
    push_colored(output, META_LABEL_STYLE, "- column:");
    output.push(' ');
    push_colored(
        output,
        META_VALUE_STYLE,
        &coordinate.column_label().to_string(),
    );
    output.push('\n');
}
fn push_to_move(board: &Board, output: &mut String) {
    push_colored(output, META_LABEL_STYLE, "To move:");
    output.push(' ');
    push_stone_name(output, board.next_stone());
    output.push('\n');
}
fn push_stone_name(output: &mut String, stone: Stone) {
    let style = match stone {
        Stone::Black => BLACK_STYLE,
        Stone::White => WHITE_STYLE,
    };
    push_colored(output, style, stone.name());
}
fn push_board(board: &Board, output: &mut String) {
    push_header(output);
    push_border(output, '┌', '┬', '┐');
    for row in 0..BOARD_SIZE {
        push_row(board, output, row);
        if row + 1 < BOARD_SIZE {
            push_border(output, '├', '┼', '┤');
        }
    }
    push_border(output, '└', '┴', '┘');
}
fn push_header(output: &mut String) {
    output.push_str("    ");
    for column in 0..BOARD_SIZE {
        if column > 0 {
            output.push(' ');
        }
        output.push(' ');
        push_colored(output, AXIS_STYLE, &axis_label(column).to_string());
        output.push(' ');
    }
    output.push('\n');
}
fn push_border(output: &mut String, left: char, separator: char, right: char) {
    output.push_str("   ");
    let mut border = String::new();
    border.push(left);
    for column in 0..BOARD_SIZE {
        if column > 0 {
            border.push(separator);
        }
        border.push_str(CELL_BORDER);
    }
    border.push(right);
    push_colored(output, GRID_STYLE, &border);
    output.push('\n');
}
fn push_row(board: &Board, output: &mut String, row: usize) {
    push_row_label(output, row);
    output.push(' ');
    push_colored(output, GRID_STYLE, "│");
    for column in 0..BOARD_SIZE {
        push_cell(board, output, row, column);
        push_colored(output, GRID_STYLE, "│");
    }
    output.push('\n');
}
fn push_row_label(output: &mut String, row: usize) {
    let label = number_label(row);
    if label.len() < 2 {
        output.push(' ');
    }
    push_colored(output, AXIS_STYLE, &label);
}
fn push_cell(board: &Board, output: &mut String, row: usize, column: usize) {
    let coordinate = match Coordinate::new(row, column) {
        Ok(value) => value,
        Err(error) => panic!("internal coordinate generation failed: {error}"),
    };
    match board.get(coordinate) {
        Some(Stone::Black) => push_padded_stone(output, BLACK_STYLE, "○"),
        Some(Stone::White) => push_padded_stone(output, WHITE_STYLE, "●"),
        None => push_colored(output, EMPTY_STYLE, EMPTY_CELL),
    }
}
fn push_padded_stone(output: &mut String, style: &str, text: &str) {
    output.push(' ');
    push_colored(output, style, text);
    output.push(' ');
}
fn push_colored(output: &mut String, style: &str, text: &str) {
    output.push_str(style);
    output.push_str(text);
    output.push_str(RESET);
}
