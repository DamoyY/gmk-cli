use crate::errors::AppError;
use crate::session::{SessionStore, Submission};
use std::process::ExitCode;
pub mod commands;
use commands::{Cli, Command, PlaceArgs, SessionArgs};
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
    let store = SessionStore::beside_executable()?;
    match cli.command {
        Command::Place(args) => place(&store, args),
        Command::Show(args) => show(&store, args),
        Command::List => list(&store),
    }
}
fn place(store: &SessionStore, args: PlaceArgs) -> Result<String, AppError> {
    let request = args.parse_request()?;
    submit_and_wait(store, &request)
}
fn show(store: &SessionStore, args: SessionArgs) -> Result<String, AppError> {
    let request = args.parse_request()?;
    let board = store.read_session_board(&request.session)?.ok_or_else(|| {
        crate::errors::InputError::UnknownSession {
            session: request.session.as_str().to_owned(),
        }
    })?;
    Ok(board.render_csv())
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
        Submission::Legal { wait_snapshot, .. } => Ok(store.wait_for_snapshot(&wait_snapshot)?),
    }
}
