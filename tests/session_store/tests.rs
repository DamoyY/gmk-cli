use gmk_cli::coordinate::Coordinate;
use gmk_cli::session::{SessionStore, Submission};
use gmk_cli::session_id::SessionId;
use gmk_cli::stone::Stone;
use std::fs;
use std::thread;
#[test]
fn session_store_persists_sessions_independently() {
    let root = temp_root("independent");
    let store = SessionStore::new(root.clone());
    let alpha = SessionId::parse("alpha").unwrap();
    let beta = SessionId::parse("beta").unwrap();
    assert!(matches!(
        store
            .submit(&alpha, Coordinate::parse("a", "a").unwrap())
            .unwrap(),
        Submission::Legal { sequence: 1, .. },
    ));
    assert!(matches!(
        store
            .submit(&beta, Coordinate::parse("a", "a").unwrap())
            .unwrap(),
        Submission::Legal { sequence: 1, .. },
    ));
    assert!(root.join("alpha.sqlite").is_file());
    assert!(root.join("beta.sqlite").is_file());
    let alpha_board = store.read_session_board(&alpha).unwrap().unwrap();
    let beta_board = store.read_session_board(&beta).unwrap().unwrap();
    assert_eq!(
        alpha_board.get(Coordinate::parse("a", "a").unwrap()),
        Some(Stone::Black),
    );
    assert_eq!(
        beta_board.get(Coordinate::parse("a", "a").unwrap()),
        Some(Stone::Black),
    );
    cleanup(root);
}
#[test]
fn illegal_moves_are_returned_immediately_without_changing_the_database() {
    let root = temp_root("illegal");
    let store = SessionStore::new(root.clone());
    let session = SessionId::parse("illegal-session").unwrap();
    let coordinate = Coordinate::parse("h", "h").unwrap();
    store.submit(&session, coordinate).unwrap();
    let result = store.submit(&session, coordinate).unwrap();
    assert!(matches!(result, Submission::Illegal(_)));
    let board = store.read_session_board(&session).unwrap().unwrap();
    assert_eq!(board.moves(), 1);
    assert_eq!(board.next_stone(), Stone::White);
    cleanup(root);
}
#[test]
fn session_names_are_saved_as_sqlite_file_names() {
    let root = temp_root("sqlite-name");
    let store = SessionStore::new(root.clone());
    let session = SessionId::parse("visible-name.1").unwrap();
    assert_result_is_err(&SessionId::parse("nested/name"));
    store
        .submit(&session, Coordinate::parse("a", "a").unwrap())
        .unwrap();
    assert!(root.join("visible-name.1.sqlite").is_file());
    cleanup(root);
}
#[test]
fn winning_move_releases_loser_snapshot_and_ends_session() {
    let root = temp_root("win");
    let store = SessionStore::new(root.clone());
    let session = SessionId::parse("win-session").unwrap();
    submit(&store, &session, "a", "a");
    submit(&store, &session, "b", "a");
    submit(&store, &session, "a", "b");
    submit(&store, &session, "b", "b");
    submit(&store, &session, "a", "c");
    submit(&store, &session, "b", "c");
    submit(&store, &session, "a", "d");
    let waiting = store
        .submit(&session, Coordinate::parse("b", "d").unwrap())
        .unwrap();
    let Submission::Legal { wait_snapshot, .. } = waiting else {
        panic!("eighth move should wait for the final move");
    };
    let won = store
        .submit(&session, Coordinate::parse("a", "e").unwrap())
        .unwrap();
    assert!(matches!(won, Submission::Won { sequence: 9 }));
    let loser_output = store.wait_for_snapshot(&wait_snapshot).unwrap();
    assert!(loser_output.starts_with(
        "Diff:\n- row: 1\n- column: e\n---\nTo move: White\n<board direction=\"0deg\">"
    ));
    assert!(loser_output.contains(" 1   X,  X,  X,  X,  X"));
    assert!(loser_output.contains("You lost."));
    let illegal = store
        .submit(&session, Coordinate::parse("c", "c").unwrap())
        .unwrap();
    assert!(matches!(illegal, Submission::Illegal(_)));
    let board = store.read_session_board(&session).unwrap().unwrap();
    assert_eq!(board.moves(), 9);
    cleanup(root);
}
#[test]
fn concurrent_valid_writes_are_serialized_without_losing_moves() {
    let root = temp_root("concurrent");
    let session = SessionId::parse("concurrent-session").unwrap();
    let mut coordinates = Vec::new();
    for row in 0_usize..4_usize {
        for column in 0_usize..4_usize {
            coordinates.push(Coordinate::new(row, column).unwrap());
        }
    }
    for column in 10_usize..14_usize {
        coordinates.push(Coordinate::new(10, column).unwrap());
    }
    let handles = coordinates
        .iter()
        .copied()
        .map(|coordinate| {
            let store = SessionStore::new(root.clone());
            let session_for_thread = session.clone();
            thread::spawn(move || store.submit(&session_for_thread, coordinate).unwrap())
        })
        .collect::<Vec<_>>();
    for handle in handles {
        assert!(matches!(handle.join().unwrap(), Submission::Legal { .. }));
    }
    let store = SessionStore::new(root.clone());
    let board = store.read_session_board(&session).unwrap().unwrap();
    assert_eq!(board.moves(), coordinates.len());
    for coordinate in coordinates {
        assert!(board.get(coordinate).is_some());
    }
    cleanup(root);
}
#[test]
fn list_sessions_returns_move_counts_in_name_order() {
    let root = temp_root("list");
    let store = SessionStore::new(root.clone());
    let alpha = SessionId::parse("alpha").unwrap();
    let beta = SessionId::parse("beta").unwrap();
    store
        .submit(&beta, Coordinate::parse("a", "a").unwrap())
        .unwrap();
    store
        .submit(&alpha, Coordinate::parse("a", "a").unwrap())
        .unwrap();
    store
        .submit(&alpha, Coordinate::parse("b", "b").unwrap())
        .unwrap();
    let summaries = store.list_sessions().unwrap();
    assert_eq!(summaries.len(), 2);
    let Some(first_summary) = summaries.first() else {
        panic!("expected first session summary");
    };
    assert_eq!(first_summary.id.as_str(), "alpha");
    assert_eq!(first_summary.moves, 2);
    let Some(second_summary) = summaries.get(1) else {
        panic!("expected second session summary");
    };
    assert_eq!(second_summary.id.as_str(), "beta");
    assert_eq!(second_summary.moves, 1);
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
fn submit(store: &SessionStore, session: &SessionId, row: &str, column: &str) {
    store
        .submit(session, Coordinate::parse(row, column).unwrap())
        .unwrap();
}
fn assert_result_is_err<T, E>(result: &Result<T, E>) {
    let is_error = result.is_err();
    assert!(is_error, "expected an error result");
}
