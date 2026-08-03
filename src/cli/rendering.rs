use super::commands::OutputFormat;
use crate::board::Board;
use crate::coordinate::Coordinate;
use crate::session::{LOSER_MESSAGE, WINNER_MESSAGE};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MoveNotice {
    None,
    Won,
    Lost,
}
pub(super) fn render_board(board: &Board, output_format: OutputFormat) -> String {
    match output_format {
        OutputFormat::Lm => board.render(),
        OutputFormat::Human => board.render_human(),
        OutputFormat::Json => board.render_json(),
        OutputFormat::Blindfold => board.render_blindfold(),
    }
}
pub(super) fn render_move(
    board: &Board,
    coordinate: Coordinate,
    output_format: OutputFormat,
    notice: MoveNotice,
) -> String {
    match output_format {
        OutputFormat::Lm => render_lm_move(board, coordinate, notice),
        OutputFormat::Human => render_human_move(board, coordinate, notice),
        OutputFormat::Json => board.render_json(),
        OutputFormat::Blindfold => render_blindfold_move(board, coordinate, notice),
    }
}
pub(super) fn render_resignation(board: &Board, output_format: OutputFormat) -> String {
    match output_format {
        OutputFormat::Lm => WINNER_MESSAGE.to_owned(),
        OutputFormat::Human => append_winner(board.render_human()),
        OutputFormat::Json => board.render_json(),
        OutputFormat::Blindfold => append_winner(board.render_blindfold()),
    }
}
fn render_lm_move(board: &Board, coordinate: Coordinate, notice: MoveNotice) -> String {
    if notice == MoveNotice::Won {
        return WINNER_MESSAGE.to_owned();
    }
    let mut output = board.render_lm_move(coordinate);
    append_plain_notice(&mut output, notice);
    output
}
fn render_human_move(board: &Board, coordinate: Coordinate, notice: MoveNotice) -> String {
    let mut output = board.render_human_move(coordinate);
    append_plain_notice(&mut output, notice);
    output
}
fn render_blindfold_move(board: &Board, coordinate: Coordinate, notice: MoveNotice) -> String {
    let mut output = board.render_blindfold_move(coordinate);
    append_plain_notice(&mut output, notice);
    output
}
fn append_winner(mut output: String) -> String {
    output.push_str(WINNER_MESSAGE);
    output
}
fn append_plain_notice(output: &mut String, notice: MoveNotice) {
    match notice {
        MoveNotice::None => {}
        MoveNotice::Won => output.push_str(WINNER_MESSAGE),
        MoveNotice::Lost => output.push_str(LOSER_MESSAGE),
    }
}
