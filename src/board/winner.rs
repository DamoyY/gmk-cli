use super::Board;
use crate::coordinate::{BOARD_SIZE, Coordinate};
use crate::stone::Stone;
const WIN_LENGTH: usize = 5;
impl Board {
    #[must_use]
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "win detection scans the board"
    )]
    pub fn winner(&self) -> Option<Stone> {
        self.horizontal_winner()
            .or_else(|| self.vertical_winner())
            .or_else(|| self.down_right_winner())
            .or_else(|| self.down_left_winner())
    }
    fn horizontal_winner(&self) -> Option<Stone> {
        for row in 0..BOARD_SIZE {
            for column in 0..=BOARD_SIZE - WIN_LENGTH {
                if let Some(stone) = self.ascending_line_winner(row, column, 0, 1) {
                    return Some(stone);
                }
            }
        }
        None
    }
    fn vertical_winner(&self) -> Option<Stone> {
        for row in 0..=BOARD_SIZE - WIN_LENGTH {
            for column in 0..BOARD_SIZE {
                if let Some(stone) = self.ascending_line_winner(row, column, 1, 0) {
                    return Some(stone);
                }
            }
        }
        None
    }
    fn down_right_winner(&self) -> Option<Stone> {
        for row in 0..=BOARD_SIZE - WIN_LENGTH {
            for column in 0..=BOARD_SIZE - WIN_LENGTH {
                if let Some(stone) = self.ascending_line_winner(row, column, 1, 1) {
                    return Some(stone);
                }
            }
        }
        None
    }
    fn down_left_winner(&self) -> Option<Stone> {
        for row in 0..=BOARD_SIZE - WIN_LENGTH {
            for column in (WIN_LENGTH - 1)..BOARD_SIZE {
                if let Some(stone) = self.descending_column_line_winner(row, column, 1, 1) {
                    return Some(stone);
                }
            }
        }
        None
    }
    fn ascending_line_winner(
        &self,
        row: usize,
        column: usize,
        row_step: usize,
        column_step: usize,
    ) -> Option<Stone> {
        let stone = self.stone_at(row, column)?;
        let complete = (1..WIN_LENGTH).all(|offset| {
            self.stone_at(row + row_step * offset, column + column_step * offset) == Some(stone)
        });
        complete.then_some(stone)
    }
    fn descending_column_line_winner(
        &self,
        row: usize,
        column: usize,
        row_step: usize,
        column_step: usize,
    ) -> Option<Stone> {
        let stone = self.stone_at(row, column)?;
        let complete = (1..WIN_LENGTH).all(|offset| {
            self.stone_at(row + row_step * offset, column - column_step * offset) == Some(stone)
        });
        complete.then_some(stone)
    }
    fn stone_at(&self, row: usize, column: usize) -> Option<Stone> {
        let coordinate = match Coordinate::new(row, column) {
            Ok(value) => value,
            Err(error) => panic!("internal coordinate generation failed: {error}"),
        };
        self.get(coordinate)
    }
}
