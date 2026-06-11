use super::{LOCK_DIR, SNAPSHOT_DIR, STATE_FILE};
use crate::room::RoomId;
use std::path::{Path, PathBuf};
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RoomPaths {
    pub(super) room_dir: PathBuf,
}
impl RoomPaths {
    pub(super) fn new(root: &Path, room: &RoomId) -> Self {
        Self {
            room_dir: root.join(room.directory_name()),
        }
    }
    fn room_dir(&self) -> &Path {
        &self.room_dir
    }
    pub(super) fn state_file(&self) -> PathBuf {
        self.room_dir().join(STATE_FILE)
    }
    pub(super) fn lock_dir(&self) -> PathBuf {
        self.room_dir.join(LOCK_DIR)
    }
    pub(super) fn snapshots_dir(&self) -> PathBuf {
        self.room_dir.join(SNAPSHOT_DIR)
    }
    pub(super) fn snapshot_file(&self, sequence: usize) -> PathBuf {
        self.snapshots_dir().join(format!("{sequence:020}.txt"))
    }
}
