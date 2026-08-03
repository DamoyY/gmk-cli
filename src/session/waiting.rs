use super::{WaitResult, WaitSnapshot, database};
use crate::errors::StorageError;
use notify::{Event, RecommendedWatcher, RecursiveMode};
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
type WatchEvent = notify::Result<Event>;
struct SnapshotWaiter {
    database_file: PathBuf,
    directory: PathBuf,
    events: Receiver<WatchEvent>,
    _watcher: RecommendedWatcher,
}
pub(super) fn wait_for_result(snapshot: &WaitSnapshot) -> Result<WaitResult, StorageError> {
    let waiter = SnapshotWaiter::new(snapshot.database_file())?;
    loop {
        if let Some(result) = database::read_wait_result(snapshot)? {
            return Ok(result);
        }
        waiter.wait_for_database_event()?;
    }
}
impl SnapshotWaiter {
    fn new(database_file: &Path) -> Result<Self, StorageError> {
        let directory = database_file
            .parent()
            .ok_or_else(|| {
                StorageError::invalid_path("watch session database directory", database_file.into())
            })?
            .to_path_buf();
        let (sender, events) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(sender).map_err(|source| {
            StorageError::notify(
                "create session database watcher",
                database_file.into(),
                source,
            )
        })?;
        notify::Watcher::watch(&mut watcher, &directory, RecursiveMode::NonRecursive).map_err(
            |source| {
                StorageError::notify(
                    "watch session database directory",
                    directory.clone(),
                    source,
                )
            },
        )?;
        Ok(Self {
            database_file: database_file.to_path_buf(),
            directory,
            events,
            _watcher: watcher,
        })
    }
    fn wait_for_database_event(&self) -> Result<(), StorageError> {
        loop {
            let received_event = self.events.recv().map_err(|source| {
                StorageError::io(
                    "receive session database notification",
                    self.database_file.clone(),
                    io::Error::new(io::ErrorKind::BrokenPipe, source),
                )
            })?;
            match received_event {
                Ok(watch_event) if self.event_mentions_database(&watch_event) => return Ok(()),
                Ok(_) => {}
                Err(source) => {
                    return Err(StorageError::notify(
                        "receive session database notification",
                        self.database_file.clone(),
                        source,
                    ));
                }
            }
        }
    }
    fn event_mentions_database(&self, event: &Event) -> bool {
        event.paths.is_empty()
            || event
                .paths
                .iter()
                .any(|path| self.path_mentions_database(path))
    }
    fn path_mentions_database(&self, path: &Path) -> bool {
        path == self.directory
            || path == self.database_file
            || file_name_starts_with(path.file_name(), self.database_file.file_name())
    }
}
fn file_name_starts_with(path_name: Option<&OsStr>, database_name: Option<&OsStr>) -> bool {
    let Some(path_file_name) = path_name.and_then(OsStr::to_str) else {
        return false;
    };
    let Some(database_file_name) = database_name.and_then(OsStr::to_str) else {
        return false;
    };
    path_file_name.starts_with(database_file_name)
}
