use super::helpers::{binary_path, spawn_place, unique_session, wait_child};
use std::process::Command;
#[test]
fn final_winning_move_returns_win_and_releases_loser() {
    let session = unique_session("win");
    let mut previous = spawn_place(&session, "1", "a");
    for (row, column) in [
        ("2", "a"),
        ("1", "b"),
        ("2", "b"),
        ("1", "c"),
        ("2", "c"),
        ("1", "d"),
        ("2", "d"),
    ] {
        let current = spawn_place(&session, row, column);
        let output = wait_child(&mut previous);
        assert!(output.is_ascii());
        previous = current;
    }
    let winner = Command::new(binary_path())
        .arg("place")
        .arg("--session")
        .arg(&session)
        .arg("1")
        .arg("e")
        .output()
        .unwrap();
    assert!(winner.status.success());
    assert_eq!(String::from_utf8(winner.stdout).unwrap(), "You win.\n");
    let loser_output = wait_child(&mut previous);
    assert!(loser_output.contains(" 1   X,  X,  X,  X,  X"));
    assert!(loser_output.contains("You lost."));
    let illegal = Command::new(binary_path())
        .arg("place")
        .arg("--session")
        .arg(&session)
        .arg("3")
        .arg("c")
        .output()
        .unwrap();
    assert!(illegal.status.success());
    let illegal_stdout = String::from_utf8(illegal.stdout).unwrap();
    assert!(illegal_stdout.contains("game is already over"));
    let shown = Command::new(binary_path())
        .arg("show")
        .arg("--session")
        .arg(&session)
        .output()
        .unwrap();
    assert!(shown.status.success());
    let shown_stdout = String::from_utf8(shown.stdout).unwrap();
    assert!(shown_stdout.contains(" 1   X,  X,  X,  X,  X"));
    assert!(!shown_stdout.contains("You lost."));
}
