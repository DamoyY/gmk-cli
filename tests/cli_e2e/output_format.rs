use super::helpers::{
    assert_still_running, binary_path, kill_child, place_then_release, spawn_place, unique_session,
    wait_child,
};
use std::process::{Command, Stdio};
#[test]
fn board_output_format_controls_place_and_show_rendering() {
    let session = unique_session("format");
    let mut waiting = Command::new(binary_path())
        .arg("place")
        .arg("--output-format")
        .arg("json")
        .arg(&session)
        .arg("a")
        .arg("a")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    assert_still_running(&mut waiting);
    let mut release = spawn_place(&session, "a", "b");
    let json_output = wait_child(&mut waiting);
    assert!(json_output.starts_with("[[\"X\",\"Y\",\"*\""));
    assert!(!json_output.contains("Diff:"));
    assert!(!json_output.contains("<board"));
    kill_child(&mut release);
    assert_human_show(&session);
    assert_human_place(&session);
}
#[test]
fn blindfold_output_format_omits_the_board() {
    let session = unique_session("blindfold");
    place_then_release(&session, ("a", "a"), ("a", "b"));
    let blindfold = Command::new(binary_path())
        .arg("show")
        .arg(&session)
        .arg("--output-format")
        .arg("blindfold")
        .output()
        .unwrap();
    assert!(blindfold.status.success());
    let blindfold_output = String::from_utf8(blindfold.stdout).unwrap();
    assert!(blindfold_output.contains("To move:"));
    assert_has_no_board(&blindfold_output);
    let mut waiting = Command::new(binary_path())
        .arg("place")
        .arg("--output-format")
        .arg("blindfold")
        .arg(&session)
        .arg("a")
        .arg("c")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    assert_still_running(&mut waiting);
    let mut release = spawn_place(&session, "a", "d");
    let output = wait_child(&mut waiting);
    assert!(output.contains("Diff:"));
    assert!(output.contains("To move:"));
    assert_has_no_board(&output);
    kill_child(&mut release);
}
fn assert_human_show(session: &str) {
    let human = Command::new(binary_path())
        .arg("show")
        .arg(session)
        .arg("--output-format")
        .arg("human")
        .output()
        .unwrap();
    assert!(human.status.success());
    let output = String::from_utf8(human.stdout).unwrap();
    assert!(output.contains("\x1b["));
    assert!(output.contains("To move:"));
    assert!(output.contains('┌'));
    assert!(output.contains('┼'));
    assert!(output.contains('○'));
    assert!(output.contains('●'));
    assert!(!output.contains('\t'));
    assert!(!output.contains("<board"));
}
fn assert_human_place(session: &str) {
    let mut waiting = Command::new(binary_path())
        .arg("place")
        .arg("--output-format")
        .arg("human")
        .arg(session)
        .arg("a")
        .arg("c")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    assert_still_running(&mut waiting);
    let mut release = spawn_place(session, "a", "d");
    let output = wait_child(&mut waiting);
    assert!(output.contains("Diff:"));
    assert!(output.contains("- row:"));
    assert!(output.contains("To move:"));
    assert!(output.contains('┌'));
    assert!(!output.contains('\t'));
    kill_child(&mut release);
}
fn assert_has_no_board(output: &str) {
    assert!(!output.contains("<board"));
    assert!(!output.contains('┌'));
    assert!(!output.contains('○'));
}
