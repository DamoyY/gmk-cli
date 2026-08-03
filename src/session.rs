use crate::board::Board;
use crate::coordinate::Coordinate;
use crate::errors::{IllegalMove, StorageError};
use crate::session_id::SessionId;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
mod database;
mod listing;
mod waiting;
use database::StoredSubmission;
pub(crate) const LOSER_MESSAGE: &str = "You lost.\n";
pub(crate) const WINNER_MESSAGE: &str = "You win.\n";
#[expect(clippy::module_name_repetitions, reason = "clearer at call sites")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionStore {
    root: PathBuf,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaitSnapshot {
    database_file: PathBuf,
    sequence: usize,
}
#[expect(clippy::exhaustive_enums, reason = "complete submission outcomes")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Submission {
    Illegal(IllegalMove),
    Legal {
        sequence: usize,
        wait_snapshot: WaitSnapshot,
    },
    Won {
        sequence: usize,
    },
}
#[expect(clippy::exhaustive_enums, reason = "complete resignation outcomes")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DefeatAdmission {
    Accepted,
    Illegal(IllegalMove),
}
#[expect(clippy::module_name_repetitions, reason = "clearer at call sites")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionSummary {
    pub id: SessionId,
    pub moves: usize,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MoveSnapshot {
    pub(crate) board: Board,
    pub(crate) coordinate: Coordinate,
    pub(crate) lost: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WaitResult {
    Move(MoveSnapshot),
    OpponentResigned(Board),
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct SessionPaths {
    database_file: PathBuf,
}
impl SessionPaths {
    #[must_use]
    #[inline]
    fn new(root: &Path, session: &SessionId) -> Self {
        Self {
            database_file: root.join(session.database_file_name()),
        }
    }
    #[must_use]
    #[inline]
    fn database_file(&self) -> &Path {
        &self.database_file
    }
}
impl WaitSnapshot {
    #[must_use]
    #[inline]
    pub(crate) const fn new(database_file: PathBuf, sequence: usize) -> Self {
        Self {
            database_file,
            sequence,
        }
    }
    #[must_use]
    #[inline]
    pub fn database_file(&self) -> &Path {
        &self.database_file
    }
    #[must_use]
    #[inline]
    pub const fn sequence(&self) -> usize {
        self.sequence
    }
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
        reason = "locked SQLite writes"
    )]
    pub fn submit(
        &self,
        session: &SessionId,
        coordinate: Coordinate,
    ) -> Result<Submission, StorageError> {
        fs::create_dir_all(&self.root).map_err(|source| {
            StorageError::io("create session directory", self.root.clone(), source)
        })?;
        let paths = SessionPaths::new(&self.root, session);
        let database_file = paths.database_file().to_path_buf();
        match database::submit(paths.database_file(), coordinate)? {
            StoredSubmission::Illegal(error) => Ok(Submission::Illegal(error)),
            StoredSubmission::Legal { sequence } => {
                let wait_sequence = next_sequence(&database_file, sequence)?;
                Ok(Submission::Legal {
                    sequence,
                    wait_snapshot: WaitSnapshot::new(database_file, wait_sequence),
                })
            }
            StoredSubmission::Won { sequence } => Ok(Submission::Won { sequence }),
        }
    }
    #[expect(clippy::missing_inline_in_public_items, reason = "locked SQLite write")]
    pub fn admit_defeat(
        &self,
        session: &SessionId,
    ) -> Result<Option<DefeatAdmission>, StorageError> {
        let paths = SessionPaths::new(&self.root, session);
        database::admit_defeat(paths.database_file())
    }
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "filesystem event waiting"
    )]
    pub fn wait_for_snapshot(&self, snapshot: &WaitSnapshot) -> Result<String, StorageError> {
        match Self::wait_for_result(snapshot)? {
            WaitResult::Move(move_snapshot) => Ok(render_lm_snapshot(&move_snapshot)),
            WaitResult::OpponentResigned(_board) => Ok(WINNER_MESSAGE.to_owned()),
        }
    }
    pub(crate) fn wait_for_result(snapshot: &WaitSnapshot) -> Result<WaitResult, StorageError> {
        waiting::wait_for_result(snapshot)
    }
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "filesystem and SQLite read boundary"
    )]
    pub fn read_session_board(&self, session: &SessionId) -> Result<Option<Board>, StorageError> {
        let paths = SessionPaths::new(&self.root, session);
        database::read_board(paths.database_file())
    }
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "filesystem enumeration and SQLite decoding"
    )]
    pub fn list_sessions(&self) -> Result<Vec<SessionSummary>, StorageError> {
        listing::collect(&self.root)
    }
}
fn render_lm_snapshot(snapshot: &MoveSnapshot) -> String {
    let mut output = snapshot.board.render_lm_move(snapshot.coordinate);
    if snapshot.lost {
        output.push_str(LOSER_MESSAGE);
    }
    output
}
fn next_sequence(path: &Path, sequence: usize) -> Result<usize, StorageError> {
    sequence.checked_add(1).ok_or_else(|| {
        StorageError::corrupt_state(path.to_path_buf(), "move sequence overflowed".to_owned())
    })
}
