use crate::errors::InputError;
const SESSION_ID_MAX_BYTES: usize = 128;
const FORBIDDEN_FILE_NAME_BYTES: &[u8] = br#"<>:"/\|?*"#;
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SessionId {
    value: String,
}
impl SessionId {
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "session parsing is validation logic rather than an accessor"
    )]
    pub fn parse(value: &str) -> Result<Self, InputError> {
        if value.is_empty() {
            return Err(InputError::EmptySession);
        }
        if value.len() > SESSION_ID_MAX_BYTES {
            return Err(InputError::SessionTooLong {
                max: SESSION_ID_MAX_BYTES,
            });
        }
        if !value.bytes().all(|byte| byte.is_ascii_graphic()) {
            return Err(InputError::NonAsciiSession);
        }
        if value
            .bytes()
            .any(|byte| FORBIDDEN_FILE_NAME_BYTES.contains(&byte))
        {
            return Err(InputError::UnsafeSessionFileName);
        }
        if is_windows_reserved_file_stem(value) {
            return Err(InputError::ReservedSessionFileName);
        }
        Ok(Self {
            value: value.to_owned(),
        })
    }
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.value
    }
    #[must_use]
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "directory name encoding allocates a new path segment"
    )]
    pub fn database_file_name(&self) -> String {
        let mut output = String::with_capacity(self.value.len() + ".sqlite".len());
        output.push_str(&self.value);
        output.push_str(".sqlite");
        output
    }
}
fn is_windows_reserved_file_stem(value: &str) -> bool {
    let stem = value.split('.').next().unwrap_or(value);
    [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ]
    .iter()
    .any(|reserved| stem.eq_ignore_ascii_case(reserved))
}
