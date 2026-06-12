use crate::coordinate::{BOARD_CELLS, BOARD_SIZE, Coordinate};
use crate::errors::IllegalMove;
use crate::stone::Stone;
use sonic_rs::to_string;
mod render;
mod winner;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Board {
    cells: [Option<Stone>; BOARD_CELLS],
    moves: usize,
    winner: Option<Stone>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlacedMove {
    pub sequence: usize,
    pub stone: Stone,
    pub won: bool,
}
impl Board {
    #[must_use]
    #[inline]
    pub const fn empty() -> Self {
        Self {
            cells: [None; BOARD_CELLS],
            moves: 0,
            winner: None,
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
        let mut board = Self {
            cells,
            moves,
            winner: None,
        };
        board.winner = board.scan_winner();
        Ok(board)
    }
    #[inline]
    pub fn place(&mut self, coordinate: Coordinate) -> Result<PlacedMove, IllegalMove> {
        if self.winner.is_some() {
            return Err(IllegalMove::GameOver);
        }
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
        let won = self.is_winning_move(coordinate, stone);
        if won {
            self.winner = Some(stone);
        }
        Ok(PlacedMove {
            sequence: self.moves,
            stone,
            won,
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
        reason = "board rendering allocates a complete JSON payload"
    )]
    pub fn render_json(&self) -> String {
        let mut output = match to_string(&json_cells(self)) {
            Ok(value) => value,
            Err(error) => panic!("internal JSON rendering failed: {error}"),
        };
        output.push('\n');
        output
    }
}
fn json_cells(board: &Board) -> [[char; BOARD_SIZE]; BOARD_SIZE] {
    core::array::from_fn(|row| core::array::from_fn(|column| json_cell_char(board, row, column)))
}
fn json_cell_char(board: &Board, row: usize, column: usize) -> char {
    let coordinate = match Coordinate::new(row, column) {
        Ok(value) => value,
        Err(error) => panic!("internal coordinate generation failed: {error}"),
    };
    board.get(coordinate).map_or('*', Stone::board_char)
}
