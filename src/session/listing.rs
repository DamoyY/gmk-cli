use super::{STATE_FILE, SessionSummary};
use crate::errors::StorageError;
use crate::persistence::decode_board;
use crate::session_id::SessionId;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
pub(super) fn collect(root: &Path) -> Result<Vec<SessionSummary>, StorageError> {
    let entries = match fs::read_dir(root) {
        Ok(value) => value,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(StorageError::io(
                "list session directory",
                root.to_path_buf(),
                error,
            ));
        }
    };
    collect_entries(entries)
}
fn collect_entries(entries: fs::ReadDir) -> Result<Vec<SessionSummary>, StorageError> {
    let mut summaries = Vec::new();
    for entry_result in entries {
        let entry = entry_result
            .map_err(|source| StorageError::io("read session entry", PathBuf::new(), source))?;
        let file_type = entry
            .file_type()
            .map_err(|source| StorageError::io("read session entry type", entry.path(), source))?;
        if !file_type.is_dir() {
            continue;
        }
        let Some(session_id) = session_id_from_directory_name(&entry.file_name())? else {
            continue;
        };
        let state_file = entry.path().join(STATE_FILE);
        let board = match fs::read_to_string(&state_file) {
            Ok(text) => decode_board(&state_file, &text)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(StorageError::io("read state", state_file, error)),
        };
        summaries.push(SessionSummary {
            id: session_id,
            moves: board.moves(),
        });
    }
    summaries.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    Ok(summaries)
}
fn session_id_from_directory_name(name: &OsStr) -> Result<Option<SessionId>, StorageError> {
    let Some(text) = name.to_str() else {
        return Ok(None);
    };
    let Some(encoded) = text.strip_prefix("session-") else {
        return Ok(None);
    };
    let decoded = decode_hex_ascii(encoded)
        .map_err(|detail| StorageError::invalid_session_directory(name, detail))?;
    SessionId::parse(&decoded)
        .map(Some)
        .map_err(|error| StorageError::invalid_session_directory(name, error.to_string()))
}
fn decode_hex_ascii(encoded: &str) -> Result<String, String> {
    let chunks = encoded.as_bytes().chunks_exact(2);
    if !chunks.remainder().is_empty() {
        return Err("encoded session id has odd length".to_owned());
    }
    let mut output = String::with_capacity(encoded.len().div_euclid(2));
    for chunk in chunks {
        let &[high_byte, low_byte] = chunk else {
            return Err("encoded session byte chunk has invalid length".to_owned());
        };
        let high = hex_value(high_byte)?;
        let low = hex_value(low_byte)?;
        let Some(value) = high.checked_mul(16).and_then(|base| base.checked_add(low)) else {
            return Err("encoded session byte overflowed".to_owned());
        };
        output.push(char::from(value));
    }
    Ok(output)
}
fn hex_value(value: u8) -> Result<u8, String> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => Err(format!("invalid hex byte '{}'", char::from(value))),
    }
}
