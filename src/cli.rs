use crate::coordinate::Coordinate;
use crate::errors::{AppError, InputError};
use crate::room::RoomId;
use crate::session::{SessionStore, Submission};
use std::env;
use std::io;
use std::process::ExitCode;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    pub room: RoomId,
    pub coordinate: Coordinate,
}
#[must_use]
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
pub fn execute() -> Result<String, AppError> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let request = if args.is_empty() {
        let mut input = String::new();
        let mut stdin = io::stdin();
        io::Read::read_to_string(&mut stdin, &mut input).map_err(|source| {
            crate::errors::StorageError::io(
                "read standard input",
                std::path::PathBuf::new(),
                source,
            )
        })?;
        parse_request_line(&input)?
    } else {
        let parts = args.iter().map(String::as_str).collect::<Vec<_>>();
        parse_request_parts(&parts)?
    };
    let store = SessionStore::beside_executable()?;
    submit_and_wait(&store, &request)
}
pub fn submit_and_wait(store: &SessionStore, request: &Request) -> Result<String, AppError> {
    match store.submit(&request.room, request.coordinate)? {
        Submission::Illegal(error) => Ok(format!("error: illegal move: {error}\n")),
        Submission::Legal { wait_snapshot, .. } => Ok(store.wait_for_snapshot(&wait_snapshot)?),
    }
}
pub fn parse_request_line(input: &str) -> Result<Request, InputError> {
    let parts = input.split_whitespace().collect::<Vec<_>>();
    parse_request_parts(&parts)
}
pub fn parse_request_parts(parts: &[&str]) -> Result<Request, InputError> {
    let mut fields = parts.iter().copied();
    let (Some(room), Some(row), Some(column), None) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return Err(InputError::ExpectedFields { found: parts.len() });
    };
    Ok(Request {
        room: RoomId::parse(room)?,
        coordinate: Coordinate::parse(row, column)?,
    })
}
