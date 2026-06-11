use super::{Board, push_plain_move_summary, push_plain_to_move};
use crate::coordinate::Coordinate;
impl Board {
    #[must_use]
    pub(crate) fn render_blindfold(&self) -> String {
        let mut output = String::new();
        push_plain_to_move(self, &mut output);
        output
    }
    #[must_use]
    pub(crate) fn render_blindfold_move(&self, coordinate: Coordinate) -> String {
        let mut output = String::new();
        push_plain_move_summary(self, &mut output, coordinate);
        output
    }
}
