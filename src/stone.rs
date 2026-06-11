#[expect(
    clippy::exhaustive_enums,
    reason = "Gomoku stones are limited to black and white"
)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stone {
    Black,
    White,
}
impl Stone {
    #[must_use]
    #[inline]
    pub const fn board_char(self) -> char {
        match self {
            Self::Black => 'X',
            Self::White => 'Y',
        }
    }
    #[must_use]
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Black => "Black",
            Self::White => "White",
        }
    }
    #[inline]
    pub const fn from_board_byte(value: u8) -> Result<Option<Self>, &'static str> {
        match value {
            b'*' => Ok(None),
            b'X' => Ok(Some(Self::Black)),
            b'Y' => Ok(Some(Self::White)),
            _ => Err("expected one of '*', 'X', or 'Y'"),
        }
    }
}
