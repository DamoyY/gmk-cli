use crate::session_id::SessionId;
use std::path::{Path, PathBuf};
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SessionPaths {
    database_file: PathBuf,
}
impl SessionPaths {
    pub(super) fn new(root: &Path, session: &SessionId) -> Self {
        Self {
            database_file: root.join(session.database_file_name()),
        }
    }
    pub(super) fn database_file(&self) -> &Path {
        &self.database_file
    }
}
