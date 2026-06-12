use super::Board;
use crate::coordinate::{BOARD_SIZE, Coordinate};
use crate::stone::Stone;
const WIN_LENGTH: usize = 5;
const SEARCH_DISTANCE: usize = WIN_LENGTH - 1;
const DIRECTIONS: [Direction; 4] = [
    Direction::new(0, 1),
    Direction::new(1, 0),
    Direction::new(1, 1),
    Direction::new(1, -1),
];
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Direction {
    row_step: isize,
    column_step: isize,
}
impl Direction {
    const fn new(row_step: isize, column_step: isize) -> Self {
        Self {
            row_step,
            column_step,
        }
    }
    fn opposite(self) -> Self {
        Self {
            row_step: checked_negate(self.row_step),
            column_step: checked_negate(self.column_step),
        }
    }
}
impl Board {
    #[must_use]
    #[inline]
    pub const fn winner(&self) -> Option<Stone> {
        self.winner
    }
    pub(super) fn scan_winner(&self) -> Option<Stone> {
        for row in 0..BOARD_SIZE {
            for column in 0..BOARD_SIZE {
                let Some(stone) = self.stone_at(row, column) else {
                    continue;
                };
                let coordinate = coordinate_from_indexes(row, column);
                if self.is_winning_move(coordinate, stone) {
                    return Some(stone);
                }
            }
        }
        None
    }
    pub(super) fn is_winning_move(&self, coordinate: Coordinate, stone: Stone) -> bool {
        DIRECTIONS
            .iter()
            .any(|direction| self.line_length(coordinate, stone, *direction) >= WIN_LENGTH)
    }
    fn line_length(&self, coordinate: Coordinate, stone: Stone, direction: Direction) -> usize {
        let forward = self.ray_length(coordinate, stone, direction);
        let backward = self.ray_length(coordinate, stone, direction.opposite());
        forward
            .checked_add(backward)
            .and_then(|partial| partial.checked_add(1))
            .unwrap_or_else(|| panic!("line length overflowed"))
    }
    fn ray_length(&self, coordinate: Coordinate, stone: Stone, direction: Direction) -> usize {
        let mut length = 0;
        let mut row = coordinate.row();
        let mut column = coordinate.column();
        while length < SEARCH_DISTANCE {
            let Some((next_row, next_column)) = next_position(row, column, direction) else {
                return length;
            };
            if self.stone_at(next_row, next_column) != Some(stone) {
                return length;
            }
            length += 1;
            row = next_row;
            column = next_column;
        }
        length
    }
    fn stone_at(&self, row: usize, column: usize) -> Option<Stone> {
        self.get(coordinate_from_indexes(row, column))
    }
}
fn next_position(row: usize, column: usize, direction: Direction) -> Option<(usize, usize)> {
    let next_row = row.checked_add_signed(direction.row_step)?;
    let next_column = column.checked_add_signed(direction.column_step)?;
    (next_row < BOARD_SIZE && next_column < BOARD_SIZE).then_some((next_row, next_column))
}
fn coordinate_from_indexes(row: usize, column: usize) -> Coordinate {
    match Coordinate::new(row, column) {
        Ok(value) => value,
        Err(error) => panic!("internal coordinate generation failed: {error}"),
    }
}
fn checked_negate(value: isize) -> isize {
    value
        .checked_neg()
        .unwrap_or_else(|| panic!("direction step overflowed"))
}
