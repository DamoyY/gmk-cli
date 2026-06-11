use super::{LOCK_DIR, SNAPSHOT_DIR, STATE_FILE};
use crate::session_id::SessionId;
use std::path::{Path, PathBuf};
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SessionPaths {
    pub(super) session_dir: PathBuf,
}
impl SessionPaths {
    pub(super) fn new(root: &Path, session: &SessionId) -> Self {
        Self {
            session_dir: root.join(session.directory_name()),
        }
    }
    fn session_dir(&self) -> &Path {
        &self.session_dir
    }
    pub(super) fn state_file(&self) -> PathBuf {
        self.session_dir().join(STATE_FILE)
    }
    pub(super) fn lock_dir(&self) -> PathBuf {
        self.session_dir.join(LOCK_DIR)
    }
    pub(super) fn snapshots_dir(&self) -> PathBuf {
        self.session_dir.join(SNAPSHOT_DIR)
    }
    pub(super) fn snapshot_file(&self, sequence: usize) -> PathBuf {
        self.snapshots_dir().join(format!("{sequence:020}.txt"))
    }
}
