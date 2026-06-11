use core::time::Duration;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Instant;
const CHILD_TIMEOUT: Duration = Duration::from_secs(10);
pub(super) const fn binary_path() -> &'static str {
    env!("CARGO_BIN_EXE_gmk-cli")
}
pub(super) fn spawn_place(session: &str, row: &str, column: &str) -> Child {
    Command::new(binary_path())
        .arg("place")
        .arg(session)
        .arg("--row")
        .arg(row)
        .arg("--column")
        .arg(column)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}
pub(super) fn place_then_release(session: &str, first: (&str, &str), second: (&str, &str)) {
    let mut waiting = spawn_place(session, first.0, first.1);
    assert_still_running(&mut waiting);
    let mut release = spawn_place(session, second.0, second.1);
    let output = wait_child(&mut waiting);
    assert!(output.is_ascii());
    kill_child(&mut release);
}
pub(super) fn assert_still_running(child: &mut Child) {
    thread::sleep(Duration::from_millis(150));
    assert!(
        child.try_wait().unwrap().is_none(),
        "child process should still be waiting for the next legal move",
    );
}
pub(super) fn wait_child(child: &mut Child) -> String {
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
pub(super) fn kill_child(child: &mut Child) {
    if child.try_wait().unwrap().is_none() {
        child.kill().unwrap();
        child.wait().unwrap();
    }
}
pub(super) fn unique_session(prefix: &str) -> String {
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
pub(super) fn label(index: usize) -> String {
    let offset = u8::try_from(index).unwrap();
    char::from(b'a' + offset).to_string()
}
