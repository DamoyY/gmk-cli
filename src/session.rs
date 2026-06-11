use crate::board::Board;
use crate::coordinate::Coordinate;
use crate::errors::{IllegalMove, StorageError};
use crate::lock::DirectoryLock;
use crate::persistence::{decode_board, encode_board};
use crate::room::RoomId;
use core::sync::atomic::{AtomicU64, Ordering};
use core::time::Duration;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
const STATE_FILE: &str = "state.txt";
const LOCK_DIR: &str = "write.lock";
const SNAPSHOT_DIR: &str = "snapshots";
const WAIT_RETRY_DELAY: Duration = Duration::from_millis(10);
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionStore {
    root: PathBuf,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Submission {
    Illegal(IllegalMove),
    Legal {
        sequence: usize,
        wait_snapshot: PathBuf,
    },
}
impl SessionStore {
    pub fn beside_executable() -> Result<Self, StorageError> {
        let executable = env::current_exe()
            .map_err(|source| StorageError::io("locate executable", PathBuf::new(), source))?;
        let parent = executable
            .parent()
            .ok_or_else(|| StorageError::NoExecutableDirectory {
                path: executable.clone(),
            })?;
        Ok(Self::new(parent.join("sessions")))
    }
    #[must_use]
    pub const fn new(root: PathBuf) -> Self {
        Self { root }
    }
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn submit(
        &self,
        room: &RoomId,
        coordinate: Coordinate,
    ) -> Result<Submission, StorageError> {
        let paths = RoomPaths::new(&self.root, room);
        fs::create_dir_all(paths.snapshots_dir()).map_err(|source| {
            StorageError::io("create session directory", paths.room_dir.clone(), source)
        })?;
        let session_lock = DirectoryLock::acquire(paths.lock_dir());
        let _session_lock = session_lock?;
        let state_file = paths.state_file();
        let mut board = Self::read_board_or_empty(&state_file)?;
        match board.place(coordinate) {
            Ok(placed) => {
                let state_text = encode_board(&board);
                write_atomic(&state_file, &state_text)?;
                let rendered = board.render_csv();
                write_atomic(&paths.snapshot_file(placed.sequence), &rendered)?;
                Ok(Submission::Legal {
                    sequence: placed.sequence,
                    wait_snapshot: paths.snapshot_file(placed.sequence + 1),
                })
            }
            Err(error) => Ok(Submission::Illegal(error)),
        }
    }
    pub fn wait_for_snapshot(&self, snapshot: &Path) -> Result<String, StorageError> {
        loop {
            match fs::read_to_string(snapshot) {
                Ok(text) => return Ok(text),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    thread::sleep(WAIT_RETRY_DELAY);
                }
                Err(error) => {
                    return Err(StorageError::io(
                        "read snapshot",
                        snapshot.to_path_buf(),
                        error,
                    ));
                }
            }
        }
    }
    pub fn read_room_board(&self, room: &RoomId) -> Result<Option<Board>, StorageError> {
        let paths = RoomPaths::new(&self.root, room);
        let state_file = paths.state_file();
        match fs::read_to_string(&state_file) {
            Ok(text) => decode_board(&state_file, &text).map(Some),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(StorageError::io("read state", state_file, error)),
        }
    }
    fn read_board_or_empty(path: &Path) -> Result<Board, StorageError> {
        match fs::read_to_string(path) {
            Ok(text) => decode_board(path, &text),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Board::empty()),
            Err(error) => Err(StorageError::io("read state", path.to_path_buf(), error)),
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct RoomPaths {
    room_dir: PathBuf,
}
impl RoomPaths {
    fn new(root: &Path, room: &RoomId) -> Self {
        Self {
            room_dir: root.join(room.directory_name()),
        }
    }
    fn room_dir(&self) -> &Path {
        &self.room_dir
    }
    fn state_file(&self) -> PathBuf {
        self.room_dir().join(STATE_FILE)
    }
    fn lock_dir(&self) -> PathBuf {
        self.room_dir.join(LOCK_DIR)
    }
    fn snapshots_dir(&self) -> PathBuf {
        self.room_dir.join(SNAPSHOT_DIR)
    }
    fn snapshot_file(&self, sequence: usize) -> PathBuf {
        self.snapshots_dir().join(format!("{sequence:020}.txt"))
    }
}
fn write_atomic(path: &Path, text: &str) -> Result<(), StorageError> {
    let parent = path.parent().ok_or_else(|| {
        StorageError::invalid_path("write file without parent", path.to_path_buf())
    })?;
    fs::create_dir_all(parent).map_err(|source| {
        StorageError::io("create parent directory", parent.to_path_buf(), source)
    })?;
    let temporary = temporary_path(path)?;
    fs::write(&temporary, text)
        .map_err(|source| StorageError::io("write temp file", temporary.clone(), source))?;
    match fs::rename(&temporary, path) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
            replace_existing_file(&temporary, path)
        }
        Err(source) => {
            if let Err(cleanup_error) = fs::remove_file(&temporary) {
                eprintln!(
                    "warning: failed to remove temp file '{}': {}",
                    crate::errors::ascii_path(&temporary),
                    cleanup_error,
                );
            }
            Err(StorageError::io("replace file", path.to_path_buf(), source))
        }
    }
}
fn replace_existing_file(temporary: &Path, path: &Path) -> Result<(), StorageError> {
    fs::remove_file(path)
        .map_err(|source| StorageError::io("remove old file", path.to_path_buf(), source))?;
    fs::rename(temporary, path)
        .map_err(|source| StorageError::io("replace file", path.to_path_buf(), source))
}
fn temporary_path(path: &Path) -> Result<PathBuf, StorageError> {
    let file_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| StorageError::invalid_path("build temp path", path.to_path_buf()))?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|source| StorageError::Clock { source })?
        .as_nanos();
    let counter = next_temp_id()?;
    Ok(path.with_file_name(format!(
        "{file_name}.{}.{}.{}.tmp",
        process::id(),
        timestamp,
        counter,
    )))
}
fn next_temp_id() -> Result<u64, StorageError> {
    NEXT_TEMP_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .map_err(|_previous_value| StorageError::TempCounterOverflow)
}
