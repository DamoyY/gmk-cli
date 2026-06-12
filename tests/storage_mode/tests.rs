use gmk_cli::coordinate::Coordinate;
use gmk_cli::session::SessionStore;
use gmk_cli::session_id::SessionId;
use rusqlite::Connection;
use std::fs;
#[test]
fn session_databases_use_write_ahead_logging() {
    let root = temp_root("wal");
    let store = SessionStore::new(root.clone());
    let session = SessionId::parse("wal-session").unwrap();
    store
        .submit(&session, Coordinate::parse("a", "a").unwrap())
        .unwrap();
    let database_file = root.join("wal-session.sqlite");
    let connection = Connection::open(database_file).unwrap();
    let journal_mode = connection
        .query_row("PRAGMA journal_mode", [], |row| row.get::<_, String>(0))
        .unwrap();
    assert_eq!(journal_mode.to_ascii_lowercase(), "wal");
    drop(connection);
    cleanup(root);
}
fn temp_root(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir()
        .join("agent")
        .join("gmk-cli-tests")
        .join(format!(
            "{}-{}-{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        ));
    fs::create_dir_all(&path).unwrap();
    path
}
fn cleanup(path: std::path::PathBuf) {
    fs::remove_dir_all(path).unwrap();
}
