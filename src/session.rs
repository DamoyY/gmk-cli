use crate::board::Board;
use crate::coordinate::Coordinate;
use crate::errors::{IllegalMove, StorageError};
use crate::lock::DirectoryLock;
use crate::persistence::{decode_board, encode_board};
use crate::session_id::SessionId;
use core::time::Duration;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
mod atomic_write;
mod listing;
mod paths;
use paths::SessionPaths;
const STATE_FILE: &str = "state.txt";
const LOCK_DIR: &str = "write.lock";
const SNAPSHOT_DIR: &str = "snapshots";
const WAIT_RETRY_DELAY: Duration = Duration::from_millis(10);
const LOSER_MESSAGE: &str = "You are lost.\n";
#[expect(clippy::module_name_repetitions, reason = "clearer at call sites")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionStore {
    root: PathBuf,
}
#[expect(clippy::exhaustive_enums, reason = "complete submission outcomes")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Submission {
    Illegal(IllegalMove),
    Legal {
        sequence: usize,
        wait_snapshot: PathBuf,
    },
    Won {
        sequence: usize,
    },
}
#[expect(clippy::module_name_repetitions, reason = "clearer at call sites")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionSummary {
    pub id: SessionId,
    pub moves: usize,
}
impl SessionStore {
    #[expect(clippy::missing_inline_in_public_items, reason = "process IO boundary")]
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
    #[inline]
    pub const fn new(root: PathBuf) -> Self {
        Self { root }
    }
    #[must_use]
    #[inline]
    pub fn root(&self) -> &Path {
        &self.root
    }
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "locked filesystem writes"
    )]
    pub fn submit(
        &self,
        session: &SessionId,
        coordinate: Coordinate,
    ) -> Result<Submission, StorageError> {
        let paths = SessionPaths::new(&self.root, session);
        fs::create_dir_all(paths.snapshots_dir()).map_err(|source| {
            StorageError::io(
                "create session directory",
                paths.session_dir.clone(),
                source,
            )
        })?;
        let _session_lock = DirectoryLock::acquire(paths.lock_dir())?;
        let state_file = paths.state_file();
        let mut board = Self::read_board_or_empty(&state_file)?;
        match board.place(coordinate) {
            Ok(placed) => {
                let state_text = encode_board(&board);
                atomic_write::write(&state_file, &state_text)?;
                let current_snapshot = paths.snapshot_file(placed.sequence);
                let rendered = if placed.won {
                    render_lost_snapshot(&board, coordinate)
                } else {
                    render_move_snapshot(&board, coordinate)
                };
                atomic_write::write(&current_snapshot, &rendered)?;
                if placed.won {
                    Ok(Submission::Won {
                        sequence: placed.sequence,
                    })
                } else {
                    Ok(Submission::Legal {
                        sequence: placed.sequence,
                        wait_snapshot: paths.snapshot_file(placed.sequence + 1),
                    })
                }
            }
            Err(error) => Ok(Submission::Illegal(error)),
        }
    }
    #[expect(clippy::missing_inline_in_public_items, reason = "filesystem polling")]
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
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "filesystem read boundary"
    )]
    pub fn read_session_board(&self, session: &SessionId) -> Result<Option<Board>, StorageError> {
        let paths = SessionPaths::new(&self.root, session);
        let state_file = paths.state_file();
        match fs::read_to_string(&state_file) {
            Ok(text) => decode_board(&state_file, &text).map(Some),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(StorageError::io("read state", state_file, error)),
        }
    }
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "filesystem enumeration and state decoding"
    )]
    pub fn list_sessions(&self) -> Result<Vec<SessionSummary>, StorageError> {
        listing::collect(&self.root)
    }
    fn read_board_or_empty(path: &Path) -> Result<Board, StorageError> {
        match fs::read_to_string(path) {
            Ok(text) => decode_board(path, &text),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Board::empty()),
            Err(error) => Err(StorageError::io("read state", path.to_path_buf(), error)),
        }
    }
}
fn render_move_snapshot(board: &Board, coordinate: Coordinate) -> String {
    let mut output = String::new();
    output.push_str("Diff:\n- row: ");
    output.push_str(&coordinate.row_label());
    output.push_str("\n- column: ");
    output.push(coordinate.column_label());
    output.push_str("\n---\nTo move: ");
    output.push_str(board.next_stone().name());
    output.push('\n');
    output.push_str(&board.render());
    output
}
fn render_lost_snapshot(board: &Board, coordinate: Coordinate) -> String {
    let mut output = render_move_snapshot(board, coordinate);
    output.push_str(LOSER_MESSAGE);
    output
}
