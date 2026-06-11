use crate::errors::InputError;
pub const BOARD_SIZE: usize = 15;
pub const BOARD_CELLS: usize = BOARD_SIZE * BOARD_SIZE;
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Coordinate {
    row: usize,
    column: usize,
}
impl Coordinate {
    #[inline]
    pub fn new(row: usize, column: usize) -> Result<Self, InputError> {
        if row < BOARD_SIZE && column < BOARD_SIZE {
            Ok(Self { row, column })
        } else {
            Err(InputError::Coordinate {
                axis: "position",
                value: format!("{row},{column}"),
                reason: "expected indexes inside a 15 by 15 board",
            })
        }
    }
    #[inline]
    pub fn parse(row: &str, column: &str) -> Result<Self, InputError> {
        let row_index = parse_axis("row", row)?;
        let column_index = parse_axis("column", column)?;
        Self::new(row_index, column_index)
    }
    #[must_use]
    #[inline]
    pub const fn row(self) -> usize {
        self.row
    }
    #[must_use]
    #[inline]
    pub const fn column(self) -> usize {
        self.column
    }
    #[must_use]
    #[inline]
    pub const fn linear_index(self) -> usize {
        self.row * BOARD_SIZE + self.column
    }
    #[must_use]
    #[inline]
    pub fn row_label(self) -> char {
        axis_label(self.row)
    }
    #[must_use]
    #[inline]
    pub fn column_label(self) -> char {
        axis_label(self.column)
    }
}
#[expect(
    clippy::missing_inline_in_public_items,
    reason = "axis parsing is validation logic rather than an accessor"
)]
pub fn parse_axis(axis: &'static str, token: &str) -> Result<usize, InputError> {
    if token.is_empty() {
        return Err(InputError::Coordinate {
            axis,
            value: token.to_owned(),
            reason: "expected a label from a to o or a number from 1 to 15",
        });
    }
    if token.len() == 1
        && let Some(byte) = token.as_bytes().first().copied()
        && (b'a'..=b'o').contains(&byte)
    {
        return Ok(usize::from(byte - b'a'));
    }
    if token.bytes().all(|byte| byte.is_ascii_digit()) {
        let parsed = match token.parse::<usize>() {
            Ok(value) => value,
            Err(error) => {
                return Err(InputError::Coordinate {
                    axis,
                    value: format!("{token}: {error}"),
                    reason: "expected a number from 1 to 15",
                });
            }
        };
        if (1..=BOARD_SIZE).contains(&parsed) {
            return Ok(parsed - 1);
        }
    }
    Err(InputError::Coordinate {
        axis,
        value: token.to_owned(),
        reason: "expected a label from a to o or a number from 1 to 15",
    })
}
#[must_use]
#[inline]
pub fn axis_label(index: usize) -> char {
    assert!(index < BOARD_SIZE, "axis index is out of range");
    let Ok(offset) = u8::try_from(index) else {
        panic!("axis index must fit in a byte");
    };
    let Some(byte) = b'a'.checked_add(offset) else {
        panic!("axis label byte overflow");
    };
    char::from(byte)
}
