use crate::board::Board;
use crate::coordinate::Coordinate;
use crate::errors::AppError;
use crate::session::{LOSER_MESSAGE, SessionStore, Submission, WaitSnapshot};
use crate::session_id::SessionId;
use std::process::ExitCode;
pub mod commands;
use commands::{Cli, Command, OutputFormat, PlaceArgs, ShowArgs};
const WINNER_MESSAGE: &str = "You win.\n";
const LLM_AGENT_PROMPT: &str = concat!(
    "Do not analyze source code, reverse engineer the program, or modify data.\n",
    "If you can set a command timeout, set it to the maximum value to avoid leaving the game early. Ideally more than 30 minutes.\n",
    "Generally, you don't need to explicitly specify `output-format`; leave it as `lm`. Only use `json` when you need to input the result into script.\n",
);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MoveNotice {
    None,
    Won,
    Lost,
}
#[must_use]
#[expect(
    clippy::missing_inline_in_public_items,
    reason = "the CLI entry point performs process IO"
)]
pub fn run() -> ExitCode {
    match execute() {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}
#[expect(
    clippy::missing_inline_in_public_items,
    reason = "request execution performs process and filesystem IO"
)]
pub fn execute() -> Result<String, AppError> {
    execute_cli(<Cli as clap::Parser>::parse())
}
fn execute_cli(cli: Cli) -> Result<String, AppError> {
    match cli.command {
        Command::Place(args) => {
            let store = SessionStore::beside_executable()?;
            place(&store, args)
        }
        Command::Show(args) => {
            let store = SessionStore::beside_executable()?;
            show(&store, args)
        }
        Command::List => {
            let store = SessionStore::beside_executable()?;
            list(&store)
        }
        Command::IAdmitDefeat => Ok(LOSER_MESSAGE.to_owned()),
        Command::ForLlmAgent => Ok(LLM_AGENT_PROMPT.to_owned()),
    }
}
fn place(store: &SessionStore, args: PlaceArgs) -> Result<String, AppError> {
    let request = args.parse_request()?;
    submit_and_wait(store, &request)
}
fn show(store: &SessionStore, args: ShowArgs) -> Result<String, AppError> {
    let request = args.parse_request()?;
    let board = read_existing_board(store, &request.session)?;
    Ok(render_board(&board, request.output_format))
}
fn list(store: &SessionStore) -> Result<String, AppError> {
    let sessions = store.list_sessions()?;
    let mut output = String::from("session,moves\n");
    for session in sessions {
        output.push_str(session.id.as_str());
        output.push(',');
        output.push_str(&session.moves.to_string());
        output.push('\n');
    }
    Ok(output)
}
#[expect(
    clippy::missing_inline_in_public_items,
    reason = "submitting requests crosses the storage boundary"
)]
pub fn submit_and_wait(
    store: &SessionStore,
    request: &commands::PlaceRequest,
) -> Result<String, AppError> {
    match store.submit(&request.session, request.coordinate)? {
        Submission::Illegal(error) => Ok(format!("error: illegal move: {error}\n")),
        Submission::Legal { wait_snapshot, .. } => {
            wait_for_formatted_snapshot(request, &wait_snapshot)
        }
        Submission::Won { .. } => render_winning_submission(store, request),
    }
}
fn wait_for_formatted_snapshot(
    request: &commands::PlaceRequest,
    wait_snapshot: &WaitSnapshot,
) -> Result<String, AppError> {
    let snapshot = SessionStore::wait_for_move_snapshot(wait_snapshot)?;
    let notice = if snapshot.lost {
        MoveNotice::Lost
    } else {
        MoveNotice::None
    };
    Ok(render_move(
        &snapshot.board,
        snapshot.coordinate,
        request.output_format,
        notice,
    ))
}
fn render_winning_submission(
    store: &SessionStore,
    request: &commands::PlaceRequest,
) -> Result<String, AppError> {
    if request.output_format == OutputFormat::Lm {
        return Ok(WINNER_MESSAGE.to_owned());
    }
    let board = read_existing_board(store, &request.session)?;
    Ok(render_move(
        &board,
        request.coordinate,
        request.output_format,
        MoveNotice::Won,
    ))
}
fn read_existing_board(store: &SessionStore, session: &SessionId) -> Result<Board, AppError> {
    store
        .read_session_board(session)?
        .ok_or_else(|| crate::errors::InputError::UnknownSession {
            session: session.as_str().to_owned(),
        })
        .map_err(AppError::from)
}
fn render_board(board: &Board, output_format: OutputFormat) -> String {
    match output_format {
        OutputFormat::Lm => board.render(),
        OutputFormat::Human => board.render_human(),
        OutputFormat::Json => board.render_json(),
        OutputFormat::Blindfold => board.render_blindfold(),
    }
}
fn render_move(
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
fn append_plain_notice(output: &mut String, notice: MoveNotice) {
    match notice {
        MoveNotice::None => {}
        MoveNotice::Won => output.push_str(WINNER_MESSAGE),
        MoveNotice::Lost => output.push_str(LOSER_MESSAGE),
    }
}
