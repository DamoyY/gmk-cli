use core::time::Duration;
use std::process::Command;
use std::time::Instant;
#[path = "game_end.rs"]
mod game_end;
#[path = "helpers.rs"]
mod helpers;
use helpers::{
    assert_still_running, binary_path, kill_child, label, place_then_release, spawn_place,
    unique_session, wait_child,
};
#[test]
fn help_commands_exit_successfully() {
    for arguments in [
        &["--help"][..],
        &["place", "--help"][..],
        &["show", "--help"][..],
        &["list", "--help"][..],
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
    assert!(stdout.contains("scripts"));
    assert!(stdout.contains("source code"));
    assert!(stdout.contains("maximum value"));
}
#[test]
fn place_rejects_positional_coordinates_and_bad_arguments() {
    let session = unique_session("bad-args");
    let positional_output = Command::new(binary_path())
        .arg("place")
        .arg(&session)
        .arg("a")
        .arg("a")
        .output()
        .unwrap();
    assert!(!positional_output.status.success());
    let missing_column_output = Command::new(binary_path())
        .arg("place")
        .arg("--session")
        .arg(&session)
        .arg("--row")
        .arg("a")
        .output()
        .unwrap();
    assert!(!missing_column_output.status.success());
}
#[test]
fn legal_place_request_waits_for_the_next_legal_session_move() {
    let session = unique_session("waits");
    let mut first = spawn_place(&session, "a", "a");
    assert_still_running(&mut first);
    let mut second = spawn_place(&session, "a", "b");
    let first_output = wait_child(&mut first);
    assert!(first_output.starts_with("Diff:\n- row: a\n- column: b\n---\nTo move: Black\n#"));
    assert!(first_output.contains("a, 0, 1, *"));
    assert_still_running(&mut second);
    let mut third = spawn_place(&session, "a", "c");
    let second_output = wait_child(&mut second);
    assert!(second_output.starts_with("Diff:\n- row: a\n- column: c\n---\nTo move: White\n#"));
    assert!(second_output.contains("a, 0, 1, 0, *"));
    kill_child(&mut third);
}
#[test]
fn illegal_place_request_returns_immediately_without_releasing_waiter() {
    let session = unique_session("illegal");
    let mut first = spawn_place(&session, "h", "h");
    assert_still_running(&mut first);
    let illegal = Command::new(binary_path())
        .arg("place")
        .arg("--session")
        .arg(&session)
        .arg("--row")
        .arg("h")
        .arg("--column")
        .arg("h")
        .output()
        .unwrap();
    assert!(illegal.status.success());
    let illegal_output = String::from_utf8(illegal.stdout).unwrap();
    assert!(illegal_output.contains("error: illegal move"));
    assert_still_running(&mut first);
    let mut release = spawn_place(&session, "h", "i");
    let first_output = wait_child(&mut first);
    assert!(first_output.contains("h, *, *, *, *, *, *, *, 0, 1"));
    kill_child(&mut release);
}
#[test]
fn show_displays_the_current_board_and_rejects_positions() {
    let session = unique_session("show");
    let mut first = spawn_place(&session, "a", "a");
    assert_still_running(&mut first);
    let mut release = spawn_place(&session, "a", "b");
    let first_output = wait_child(&mut first);
    assert!(first_output.contains("a, 0, 1, *"));
    kill_child(&mut release);
    let output = Command::new(binary_path())
        .arg("show")
        .arg(&session)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("a, 0, 1, *"));
    let show_with_position_output = Command::new(binary_path())
        .arg("show")
        .arg(&session)
        .arg("a")
        .output()
        .unwrap();
    assert!(!show_with_position_output.status.success());
}
#[test]
fn list_displays_sessions_and_move_counts() {
    let first_session = unique_session("list-alpha");
    let second_session = unique_session("list-beta");
    place_then_release(&first_session, ("a", "a"), ("a", "b"));
    place_then_release(&second_session, ("b", "a"), ("b", "b"));
    place_then_release(&second_session, ("c", "a"), ("c", "b"));
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
    let mut previous = spawn_place(&session, "a", "a");
    let move_count: usize = 64;
    for index in 1..move_count {
        let row = index.div_euclid(15_usize);
        let column = index.rem_euclid(15_usize);
        let current = spawn_place(&session, &label(row), &label(column));
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
