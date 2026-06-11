#![expect(
    clippy::tests_outside_test_module,
    reason = "integration tests stay flat to honor the no inline_modules requirement"
)]
use core::time::Duration;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Instant;
const CHILD_TIMEOUT: Duration = Duration::from_secs(10);
#[test]
fn stdin_request_is_valid_and_bad_argument_count_is_rejected() {
    let room = unique_room("stdin");
    let mut first = spawn_with_stdin(&room, "a", "a");
    assert_still_running(&mut first);
    let mut second = spawn_with_stdin(&room, "a", "b");
    let first_output = wait_child(&mut first);
    assert!(first_output.contains("a, 0, 1, *"));
    kill_child(&mut second);
    let output = Command::new(binary_path())
        .arg("too")
        .arg("few")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("expected three fields")
    );
}
#[test]
fn legal_cli_request_waits_for_the_next_legal_room_move() {
    let room = unique_room("waits");
    let mut first = spawn_args(&room, "a", "a");
    assert_still_running(&mut first);
    let mut second = spawn_args(&room, "a", "b");
    let first_output = wait_child(&mut first);
    assert!(first_output.contains("a, 0, 1, *"));
    assert_still_running(&mut second);
    let mut third = spawn_args(&room, "a", "c");
    let second_output = wait_child(&mut second);
    assert!(second_output.contains("a, 0, 1, 0, *"));
    kill_child(&mut third);
}
#[test]
fn illegal_cli_request_returns_immediately_without_releasing_waiter() {
    let room = unique_room("illegal");
    let mut first = spawn_args(&room, "h", "h");
    assert_still_running(&mut first);
    let illegal = Command::new(binary_path())
        .arg(&room)
        .arg("h")
        .arg("h")
        .output()
        .unwrap();
    assert!(illegal.status.success());
    let illegal_output = String::from_utf8(illegal.stdout).unwrap();
    assert!(illegal_output.contains("error: illegal move"));
    assert_still_running(&mut first);
    let mut release = spawn_args(&room, "h", "i");
    let first_output = wait_child(&mut first);
    assert!(first_output.contains("h, *, *, *, *, *, *, *, 0, 1"));
    kill_child(&mut release);
}
#[test]
fn complete_e2e_performance_timing_test() {
    let room = unique_room("perf");
    let start = Instant::now();
    let mut previous = spawn_args(&room, "a", "a");
    let move_count: usize = 64;
    for index in 1..move_count {
        let row = index.div_euclid(15_usize);
        let column = index.rem_euclid(15_usize);
        let current = spawn_args(&room, &label(row), &label(column));
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
const fn binary_path() -> &'static str {
    env!("CARGO_BIN_EXE_gmk-cli")
}
fn spawn_args(room: &str, row: &str, column: &str) -> Child {
    Command::new(binary_path())
        .arg(room)
        .arg(row)
        .arg(column)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}
fn spawn_with_stdin(room: &str, row: &str, column: &str) -> Child {
    let mut child = Command::new(binary_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let mut stdin = child.stdin.take().unwrap();
        let input = format!("{room} {row} {column}\n");
        std::io::Write::write_all(&mut stdin, input.as_bytes()).unwrap();
    }
    child
}
fn assert_still_running(child: &mut Child) {
    thread::sleep(Duration::from_millis(150));
    assert!(
        child.try_wait().unwrap().is_none(),
        "child process should still be waiting for the next legal move",
    );
}
fn wait_child(child: &mut Child) -> String {
    let start = Instant::now();
    loop {
        if child.try_wait().unwrap().is_some() {
            let mut output = String::new();
            let stdout = child.stdout.as_mut().unwrap();
            std::io::Read::read_to_string(stdout, &mut output).unwrap();
            return output;
        }
        assert!(start.elapsed() < CHILD_TIMEOUT, "child process timed out");
        thread::sleep(Duration::from_millis(10));
    }
}
fn kill_child(child: &mut Child) {
    if child.try_wait().unwrap().is_none() {
        child.kill().unwrap();
        child.wait().unwrap();
    }
}
fn unique_room(prefix: &str) -> String {
    format!(
        "{}-{}-{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    )
}
fn label(index: usize) -> String {
    let offset = u8::try_from(index).unwrap();
    char::from(b'a' + offset).to_string()
}
