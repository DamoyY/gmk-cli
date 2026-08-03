use super::helpers::{assert_still_running, binary_path, spawn_place, unique_session, wait_child};
use std::process::{Command, Stdio};
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
#[test]
fn admitting_defeat_releases_opponents_waiting_command() {
    let session = unique_session("resignation");
    let mut opponent = spawn_place(&session, "1", "a");
    assert_still_running(&mut opponent);
    let defeat = Command::new(binary_path())
        .arg("i-admit-defeat")
        .arg("--session")
        .arg(&session)
        .output()
        .unwrap();
    assert!(defeat.status.success());
    assert_eq!(String::from_utf8(defeat.stdout).unwrap(), "You lost.\n");
    assert_eq!(wait_child(&mut opponent), "You win.\n");
    let illegal = Command::new(binary_path())
        .arg("place")
        .arg("--session")
        .arg(&session)
        .arg("1")
        .arg("b")
        .output()
        .unwrap();
    assert!(illegal.status.success());
    assert_eq!(
        String::from_utf8(illegal.stdout).unwrap(),
        "error: illegal move: game is already over\n",
    );
    let repeated = Command::new(binary_path())
        .arg("i-admit-defeat")
        .arg("--session")
        .arg(&session)
        .output()
        .unwrap();
    assert!(repeated.status.success());
    assert_eq!(
        String::from_utf8(repeated.stdout).unwrap(),
        "error: illegal move: game is already over\n",
    );
}
#[test]
fn admitting_defeat_does_not_create_an_unknown_session() {
    let session = unique_session("unknown-resignation");
    let defeat = Command::new(binary_path())
        .arg("i-admit-defeat")
        .arg("--session")
        .arg(&session)
        .output()
        .unwrap();
    assert!(!defeat.status.success());
    let stderr = String::from_utf8(defeat.stderr).unwrap();
    assert!(stderr.contains("does not exist"));
    let sessions = Command::new(binary_path()).arg("list").output().unwrap();
    assert!(sessions.status.success());
    assert!(
        !String::from_utf8(sessions.stdout)
            .unwrap()
            .contains(&session)
    );
}
#[test]
fn resignation_preserves_json_output_for_the_waiting_command() {
    let session = unique_session("json-resignation");
    let mut opponent = Command::new(binary_path())
        .arg("place")
        .arg("--session")
        .arg(&session)
        .arg("--output-format")
        .arg("json")
        .arg("1")
        .arg("a")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    assert_still_running(&mut opponent);
    let defeat = Command::new(binary_path())
        .arg("i-admit-defeat")
        .arg("--session")
        .arg(&session)
        .output()
        .unwrap();
    assert!(defeat.status.success());
    let output = wait_child(&mut opponent);
    assert!(output.starts_with("[[\"X\""));
    assert!(output.ends_with("]]\n"));
    assert!(!output.contains("You win."));
}
