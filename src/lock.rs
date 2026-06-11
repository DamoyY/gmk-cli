use crate::errors::{StorageError, ascii_path};
use core::time::Duration;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::thread;
const LOCK_RETRY_DELAY: Duration = Duration::from_millis(10);
#[expect(
    clippy::module_name_repetitions,
    reason = "DirectoryLock is clearer than Directory in call sites"
)]
#[derive(Debug)]
pub struct DirectoryLock {
    path: PathBuf,
}
impl DirectoryLock {
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "filesystem lock acquisition is not an inlining boundary"
    )]
    pub fn acquire(path: PathBuf) -> Result<Self, StorageError> {
        loop {
            #[expect(
                clippy::create_dir,
                reason = "atomic create_dir is the lock acquisition primitive"
            )]
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    thread::sleep(LOCK_RETRY_DELAY);
                }
                Err(error) => {
                    return Err(StorageError::io("create lock directory", path, error));
                }
            }
        }
    }
}
#[expect(
    clippy::missing_trait_methods,
    reason = "Drop only needs the stable drop hook to release the directory lock"
)]
impl Drop for DirectoryLock {
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "drop performs filesystem cleanup and warning output"
    )]
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir(&self.path) {
            eprintln!(
                "warning: failed to release lock directory '{}': {}",
                ascii_path(&self.path),
                error,
            );
        }
    }
}
