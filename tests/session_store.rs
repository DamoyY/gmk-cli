#![expect(
    clippy::tests_outside_test_module,
    reason = "integration tests stay flat to honor the no inline_modules requirement"
)]
use gmk_cli::coordinate::Coordinate;
use gmk_cli::room::RoomId;
use gmk_cli::session::{SessionStore, Submission};
use gmk_cli::stone::Stone;
use std::fs;
use std::thread;
#[test]
fn session_store_persists_rooms_independently() {
    let root = temp_root("independent");
    let store = SessionStore::new(root.clone());
    let alpha = RoomId::parse("alpha").unwrap();
    let beta = RoomId::parse("beta").unwrap();
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
    let alpha_board = store.read_room_board(&alpha).unwrap().unwrap();
    let beta_board = store.read_room_board(&beta).unwrap().unwrap();
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
fn illegal_moves_are_returned_immediately_and_do_not_create_snapshots() {
    let root = temp_root("illegal");
    let store = SessionStore::new(root.clone());
    let room = RoomId::parse("illegal-room").unwrap();
    let coordinate = Coordinate::parse("h", "h").unwrap();
    store.submit(&room, coordinate).unwrap();
    let result = store.submit(&room, coordinate).unwrap();
    assert!(matches!(result, Submission::Illegal(_)));
    let board = store.read_room_board(&room).unwrap().unwrap();
    assert_eq!(board.moves(), 1);
    assert_eq!(board.next_stone(), Stone::White);
    cleanup(root);
}
#[test]
fn concurrent_valid_writes_are_serialized_without_losing_moves() {
    let root = temp_root("concurrent");
    let room = RoomId::parse("concurrent-room").unwrap();
    let coordinates = (0_usize..20_usize)
        .map(|index| Coordinate::new(index.div_euclid(5_usize), index.rem_euclid(5_usize)).unwrap())
        .collect::<Vec<_>>();
    let handles = coordinates
        .iter()
        .copied()
        .map(|coordinate| {
            let store = SessionStore::new(root.clone());
            let room_for_thread = room.clone();
            thread::spawn(move || store.submit(&room_for_thread, coordinate).unwrap())
        })
        .collect::<Vec<_>>();
    for handle in handles {
        assert!(matches!(handle.join().unwrap(), Submission::Legal { .. }));
    }
    let store = SessionStore::new(root.clone());
    let board = store.read_room_board(&room).unwrap().unwrap();
    assert_eq!(board.moves(), coordinates.len());
    for coordinate in coordinates {
        assert!(board.get(coordinate).is_some());
    }
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
