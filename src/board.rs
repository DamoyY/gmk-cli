use crate::coordinate::{BOARD_CELLS, BOARD_SIZE, Coordinate, axis_label};
use crate::errors::IllegalMove;
use crate::stone::Stone;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Board {
    cells: [Option<Stone>; BOARD_CELLS],
    moves: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlacedMove {
    pub sequence: usize,
    pub stone: Stone,
}
impl Board {
    #[must_use]
    #[inline]
    pub const fn empty() -> Self {
        Self {
            cells: [None; BOARD_CELLS],
            moves: 0,
        }
    }
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "board validation is not a useful inlining boundary"
    )]
    pub fn from_parts(cells: [Option<Stone>; BOARD_CELLS], moves: usize) -> Result<Self, String> {
        if moves > BOARD_CELLS {
            return Err("move count is greater than board capacity".to_owned());
        }
        let black_count = cells
            .iter()
            .filter(|cell| matches!(cell, Some(Stone::Black)))
            .count();
        let white_count = cells
            .iter()
            .filter(|cell| matches!(cell, Some(Stone::White)))
            .count();
        let occupied = black_count + white_count;
        if occupied != moves {
            return Err(format!(
                "move count {moves} does not match occupied cells {occupied}",
            ));
        }
        let expected_black = moves.div_ceil(2);
        let expected_white = moves.div_euclid(2);
        if black_count != expected_black || white_count != expected_white {
            return Err(format!(
                "stone counts are invalid for alternating play: black {black_count}, white {white_count}",
            ));
        }
        Ok(Self { cells, moves })
    }
    #[inline]
    pub fn place(&mut self, coordinate: Coordinate) -> Result<PlacedMove, IllegalMove> {
        if self.moves == BOARD_CELLS {
            return Err(IllegalMove::BoardFull);
        }
        let stone = self.next_stone();
        let index = coordinate.linear_index();
        let Some(cell) = self.cells.get_mut(index) else {
            return Err(IllegalMove::InvalidCoordinate);
        };
        if cell.is_some() {
            return Err(IllegalMove::Occupied { coordinate });
        }
        *cell = Some(stone);
        self.moves += 1;
        Ok(PlacedMove {
            sequence: self.moves,
            stone,
        })
    }
    #[must_use]
    #[inline]
    pub const fn moves(&self) -> usize {
        self.moves
    }
    #[must_use]
    #[inline]
    pub fn get(&self, coordinate: Coordinate) -> Option<Stone> {
        let Some(cell) = self.cells.get(coordinate.linear_index()) else {
            panic!("coordinate index must be inside board");
        };
        *cell
    }
    #[must_use]
    #[inline]
    pub const fn cells(&self) -> &[Option<Stone>; BOARD_CELLS] {
        &self.cells
    }
    #[must_use]
    #[inline]
    pub const fn next_stone(&self) -> Stone {
        if self.moves & 1 == 0 {
            Stone::Black
        } else {
            Stone::White
        }
    }
    #[must_use]
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "CSV rendering is a formatting routine, not a hot accessor"
    )]
    pub fn render_csv(&self) -> String {
        let mut output = String::new();
        output.push('#');
        for column in 0..BOARD_SIZE {
            output.push_str(", ");
            output.push(axis_label(column));
        }
        for row in 0..BOARD_SIZE {
            output.push('\n');
            output.push(axis_label(row));
            for column in 0..BOARD_SIZE {
                output.push_str(", ");
                let coordinate = match Coordinate::new(row, column) {
                    Ok(value) => value,
                    Err(error) => panic!("internal coordinate generation failed: {error}"),
                };
                output.push(self.get(coordinate).map_or('*', Stone::board_char));
            }
        }
        output.push('\n');
        output
    }
}
