use core::time::Duration;
use std::process::Command;
use std::time::Instant;
#[path = "game_end.rs"]
mod game_end;
#[path = "helpers.rs"]
mod helpers;
#[path = "output_format.rs"]
mod output_format;
use helpers::{
    assert_still_running, binary_path, column_label, kill_child, place_then_release, row_label,
    spawn_place, unique_session, wait_child,
};
#[test]
fn help_commands_exit_successfully() {
    for arguments in [
        &["--help"][..],
        &["place", "--help"][..],
        &["show", "--help"][..],
        &["list", "--help"][..],
        &["i-admit-defeat", "--help"][..],
        &["for-llm-agent", "--help"][..],
    ] {
        let output = Command::new(binary_path())
            .args(arguments)
            .output()
            .unwrap();
        assert!(output.status.success());
    }
}
#[test]
fn for_llm_agent_prints_safe_play_prompt() {
    let output = Command::new(binary_path())
        .arg("for-llm-agent")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("source code"));
    assert!(stdout.contains("maximum value"));
}
#[test]
fn i_admit_defeat_prints_loss_message() {
    let output = Command::new(binary_path())
        .arg("i-admit-defeat")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "You lost.\n");
}
#[test]
fn place_accepts_unlabeled_coordinates_and_rejects_bad_arguments() {
    let session = unique_session("bad-args");
    let mut numeric_first = spawn_place(&session, "1", "a");
    assert_still_running(&mut numeric_first);
    let mut release = spawn_place(&session, "b", "1");
    let numeric_first_output = wait_child(&mut numeric_first);
    assert!(numeric_first_output.contains(" 1   X,  Y,  *"));
    kill_child(&mut release);
    let positional_session_output = Command::new(binary_path())
        .arg("place")
        .arg(&session)
        .arg("1")
        .arg("a")
        .output()
        .unwrap();
    assert!(!positional_session_output.status.success());
    let row_option_output = Command::new(binary_path())
        .arg("place")
        .arg("--session")
        .arg(&session)
        .arg("--row")
        .arg("1")
        .output()
        .unwrap();
    assert!(!row_option_output.status.success());
}
#[test]
fn legal_place_request_waits_for_the_next_legal_session_move() {
    let session = unique_session("waits");
    let mut first = spawn_place(&session, "1", "a");
    assert_still_running(&mut first);
    let mut second = spawn_place(&session, "1", "b");
    let first_output = wait_child(&mut first);
    assert!(first_output.starts_with(
        "Diff:\n- row: 1\n- column: b\n---\nTo move: Black\n<board direction=\"0deg\">"
    ));
    assert!(first_output.contains(" 1   X,  Y,  *"));
    assert_still_running(&mut second);
    let mut third = spawn_place(&session, "c", "1");
    let second_output = wait_child(&mut second);
    assert!(second_output.starts_with(
        "Diff:\n- row: 1\n- column: c\n---\nTo move: White\n<board direction=\"0deg\">"
    ));
    assert!(second_output.contains(" 1   X,  Y,  X,  *"));
    kill_child(&mut third);
}
#[test]
fn illegal_place_request_returns_immediately_without_releasing_waiter() {
    let session = unique_session("illegal");
    let mut first = spawn_place(&session, "8", "h");
    assert_still_running(&mut first);
    let illegal = Command::new(binary_path())
        .arg("place")
        .arg("--session")
        .arg(&session)
        .arg("8")
        .arg("h")
        .output()
        .unwrap();
    assert!(illegal.status.success());
    let illegal_output = String::from_utf8(illegal.stdout).unwrap();
    assert!(illegal_output.contains("error: illegal move"));
    assert_still_running(&mut first);
    let mut release = spawn_place(&session, "8", "i");
    let first_output = wait_child(&mut first);
    assert!(first_output.contains(" 8   *,  *,  *,  *,  *,  *,  *,  X,  Y"));
    kill_child(&mut release);
}
#[test]
fn show_displays_the_current_board_and_rejects_positions() {
    let session = unique_session("show");
    let mut first = spawn_place(&session, "1", "a");
    assert_still_running(&mut first);
    let mut release = spawn_place(&session, "1", "b");
    let first_output = wait_child(&mut first);
    assert!(first_output.contains(" 1   X,  Y,  *"));
    kill_child(&mut release);
    let output = Command::new(binary_path())
        .arg("show")
        .arg("--session")
        .arg(&session)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(" 1   X,  Y,  *"));
    let show_with_position_output = Command::new(binary_path())
        .arg("show")
        .arg("--session")
        .arg(&session)
        .arg("a")
        .output()
        .unwrap();
    assert!(!show_with_position_output.status.success());
    let positional_session_output = Command::new(binary_path())
        .arg("show")
        .arg(&session)
        .output()
        .unwrap();
    assert!(!positional_session_output.status.success());
}
#[test]
fn list_displays_sessions_and_move_counts() {
    let first_session = unique_session("list-alpha");
    let second_session = unique_session("list-beta");
    place_then_release(&first_session, ("1", "a"), ("1", "b"));
    place_then_release(&second_session, ("2", "a"), ("2", "b"));
    place_then_release(&second_session, ("3", "a"), ("3", "b"));
    let output = Command::new(binary_path()).arg("list").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("session,moves"));
    assert!(stdout.contains(&format!("{first_session},2")));
    assert!(stdout.contains(&format!("{second_session},4")));
}
#[test]
fn complete_e2e_performance_timing_test() {
    let session = unique_session("perf");
    let start = Instant::now();
    let mut previous = spawn_place(&session, "1", "a");
    let move_count: usize = 64;
    for index in 1..move_count {
        let row = index.div_euclid(15_usize);
        let column = index.rem_euclid(15_usize);
        let current = spawn_place(&session, &row_label(row), &column_label(column));
        let output = wait_child(&mut previous);
        assert!(output.is_ascii());
        previous = current;
    }
    kill_child(&mut previous);
    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();
    eprintln!("e2e performance: {move_count} legal CLI requests completed in {elapsed_ms} ms");
    assert!(
        elapsed < Duration::from_secs(20),
        "e2e timing exceeded limit: {elapsed_ms} ms",
    );
}
