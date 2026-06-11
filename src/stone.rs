#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stone {
    Black,
    White,
}
impl Stone {
    #[must_use]
    pub const fn board_char(self) -> char {
        match self {
            Self::Black => '0',
            Self::White => '1',
        }
    }
    pub const fn from_board_byte(value: u8) -> Result<Option<Self>, &'static str> {
        match value {
            b'*' => Ok(None),
            b'0' => Ok(Some(Self::Black)),
            b'1' => Ok(Some(Self::White)),
            _ => Err("expected one of '*', '0', or '1'"),
        }
    }
}
