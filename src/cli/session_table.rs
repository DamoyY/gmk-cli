use crate::session::SessionSummary;
pub(super) fn render(sessions: &[SessionSummary]) -> String {
    let mut output = String::with_capacity(output_capacity(sessions));
    output.push_str("session,moves\n");
    for session in sessions {
        output.push_str(session.id.as_str());
        output.push(',');
        output.push_str(&session.moves.to_string());
        output.push('\n');
    }
    output
}
fn output_capacity(sessions: &[SessionSummary]) -> usize {
    let mut capacity = "session,moves\n".len();
    for session in sessions {
        capacity += session.id.as_str().len();
        capacity += 1;
        capacity += decimal_digit_count(session.moves);
        capacity += 1;
    }
    capacity
}
const fn decimal_digit_count(value: usize) -> usize {
    let mut digits = 1;
    let mut remaining = value;
    while remaining >= 10 {
        remaining /= 10;
        digits += 1;
    }
    digits
}
