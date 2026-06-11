use std::path::Path;
#[must_use]
pub(super) fn escape(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        for escaped in core::ascii::escape_default(byte) {
            output.push(char::from(escaped));
        }
    }
    output
}
#[must_use]
pub(super) fn path(path: &Path) -> String {
    escape(&path.to_string_lossy())
}
