use crate::errors::InputError;
pub const ROOM_ID_MAX_BYTES: usize = 128;
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RoomId {
    value: String,
}
impl RoomId {
    pub fn parse(value: &str) -> Result<Self, InputError> {
        if value.is_empty() {
            return Err(InputError::EmptyRoom);
        }
        if value.len() > ROOM_ID_MAX_BYTES {
            return Err(InputError::RoomTooLong {
                max: ROOM_ID_MAX_BYTES,
            });
        }
        if !value.bytes().all(|byte| byte.is_ascii_graphic()) {
            return Err(InputError::NonAsciiRoom);
        }
        Ok(Self {
            value: value.to_owned(),
        })
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
    #[must_use]
    pub fn directory_name(&self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(5 + self.value.len() * 2);
        output.push_str("room-");
        for byte in self.value.bytes() {
            output.push(hex_char(HEX, usize::from(byte >> 4_u8)));
            output.push(hex_char(HEX, usize::from(byte & 0x0f_u8)));
        }
        output
    }
}
fn hex_char(hex: &[u8; 16], index: usize) -> char {
    let Some(value) = hex.get(index).copied() else {
        panic!("hex index must be inside the lookup table");
    };
    char::from(value)
}
