use super::Board;
use crate::coordinate::{BOARD_SIZE, Coordinate, axis_label, number_label};
use crate::stone::Stone;
const EMPTY_CELL: char = '*';
const ITEM_SEPARATOR: &str = ", ";
const AXIS_GAP: &str = "  ";
const AXIS_WIDTH: usize = 2;
impl Board {
    #[must_use]
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "board rendering is formatting, not a hot accessor"
    )]
    pub fn render(&self) -> String {
        let mut output = String::new();
        self.push_tagged_board(&mut output, "0deg", Self::render_zero_degrees);
        self.push_tagged_board(&mut output, "45deg-clockwise", Self::render_clockwise_45);
        self.push_tagged_board(&mut output, "90deg-clockwise", Self::render_clockwise_90);
        self.push_tagged_board(&mut output, "135deg-clockwise", Self::render_clockwise_135);
        output
    }
    fn push_tagged_board(
        &self,
        output: &mut String,
        direction: &'static str,
        renderer: fn(&Self) -> String,
    ) {
        output.push_str("<board direction=\"");
        output.push_str(direction);
        output.push_str("\">\n");
        output.push_str(&renderer(self));
        output.push_str("</board>\n");
    }
    fn render_zero_degrees(&self) -> String {
        let mut output = String::new();
        push_axis_header(
            &mut output,
            (0..BOARD_SIZE).map(|column| axis_label(column).to_string()),
        );
        for row in 0..BOARD_SIZE {
            output.push('\n');
            push_axis_label(&mut output, &number_label(row));
            for column in 0..BOARD_SIZE {
                push_board_item(&mut output, self.cell_char(row, column));
            }
        }
        output.push('\n');
        output
    }
    fn render_clockwise_45(&self) -> String {
        let mut output = String::new();
        for start_column in 0..BOARD_SIZE {
            self.push_down_left_diagonal(&mut output, 0, start_column);
        }
        for start_row in 1..BOARD_SIZE {
            self.push_down_left_diagonal(&mut output, start_row, BOARD_SIZE - 1);
        }
        output
    }
    fn render_clockwise_90(&self) -> String {
        let mut output = String::new();
        push_axis_header(&mut output, (0..BOARD_SIZE).rev().map(number_label));
        for row in 0..BOARD_SIZE {
            output.push('\n');
            push_axis_label(&mut output, &axis_label(row).to_string());
            for column in 0..BOARD_SIZE {
                let original_row = BOARD_SIZE - 1 - column;
                let original_column = row;
                push_board_item(&mut output, self.cell_char(original_row, original_column));
            }
        }
        output.push('\n');
        output
    }
    fn render_clockwise_135(&self) -> String {
        let mut output = String::new();
        for start_column in (0..BOARD_SIZE).rev() {
            self.push_down_right_diagonal(&mut output, 0, start_column);
        }
        for start_row in 1..BOARD_SIZE {
            self.push_down_right_diagonal(&mut output, start_row, 0);
        }
        output
    }
    fn push_down_left_diagonal(&self, output: &mut String, start_row: usize, start_column: usize) {
        let length = down_left_length(start_row, start_column);
        push_rotated_indent(output, length);
        for offset in 0..length {
            if offset > 0 {
                output.push_str(ITEM_SEPARATOR);
            }
            output.push(self.cell_char(start_row + offset, start_column - offset));
        }
        output.push('\n');
    }
    fn push_down_right_diagonal(&self, output: &mut String, start_row: usize, start_column: usize) {
        let length = down_right_length(start_row, start_column);
        push_rotated_indent(output, length);
        for offset in 0..length {
            if offset > 0 {
                output.push_str(ITEM_SEPARATOR);
            }
            output.push(self.cell_char(start_row + offset, start_column + offset));
        }
        output.push('\n');
    }
    fn cell_char(&self, row: usize, column: usize) -> char {
        let coordinate = match Coordinate::new(row, column) {
            Ok(value) => value,
            Err(error) => panic!("internal coordinate generation failed: {error}"),
        };
        self.get(coordinate).map_or(EMPTY_CELL, Stone::board_char)
    }
}
fn push_axis_header(output: &mut String, labels: impl Iterator<Item = String>) {
    output.push_str("    ");
    for label in labels {
        push_board_text(output, &label);
    }
}
fn push_axis_label(output: &mut String, label: &str) {
    push_fixed_width(output, label);
    output.push_str(AXIS_GAP);
}
fn push_board_item(output: &mut String, item: char) {
    push_board_text(output, &item.to_string());
}
fn push_board_text(output: &mut String, text: &str) {
    if !output.ends_with(AXIS_GAP) && !output.ends_with("    ") {
        output.push_str(ITEM_SEPARATOR);
    }
    push_fixed_width(output, text);
}
fn push_fixed_width(output: &mut String, text: &str) {
    let padding = AXIS_WIDTH.saturating_sub(text.len());
    for _ in 0..padding {
        output.push(' ');
    }
    output.push_str(text);
}
fn down_left_length(start_row: usize, start_column: usize) -> usize {
    let rows_available = BOARD_SIZE - start_row;
    let columns_available = start_column + 1;
    rows_available.min(columns_available)
}
fn down_right_length(start_row: usize, start_column: usize) -> usize {
    let rows_available = BOARD_SIZE - start_row;
    let columns_available = BOARD_SIZE - start_column;
    rows_available.min(columns_available)
}
fn push_rotated_indent(output: &mut String, cell_count: usize) {
    let max_width = rotated_width(BOARD_SIZE);
    let line_width = rotated_width(cell_count);
    let indent = (max_width - line_width).div_euclid(2);
    for _ in 0..indent {
        output.push(' ');
    }
}
const fn rotated_width(cell_count: usize) -> usize {
    if cell_count == 0 {
        return 0;
    }
    cell_count + ITEM_SEPARATOR.len() * (cell_count - 1)
}
