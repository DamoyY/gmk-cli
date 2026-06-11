use crate::coordinate::Coordinate;
use crate::errors::InputError;
use crate::session_id::SessionId;
use clap::{Args, Parser, Subcommand};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaceRequest {
    pub session: SessionId,
    pub coordinate: Coordinate,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShowRequest {
    pub session: SessionId,
}
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Play Gomoku with a bot in the terminal.\n\nX: Black\nY: White\n\nYour opponent may be slower, please be patient.",
    arg_required_else_help = true,
    subcommand_required = true,
    after_help = "Examples:\n  gmk-cli place demo 8 h\n  gmk-cli show demo\n  gmk-cli list"
)]
pub struct Cli {
    #[command(subcommand)]
    pub(super) command: Command,
}
#[derive(Subcommand, Debug)]
pub(super) enum Command {
    #[command(
        about = "Place stone on an existing or new board.",
        after_help = "Examples:\n  gmk-cli place demo 1 a\n  gmk-cli place --session demo --row 8 --column h"
    )]
    Place(PlaceArgs),
    #[command(
        about = "Show the board.",
        long_about = "Show the current board for a session without placing a stone.",
        after_help = "Examples:\n  gmk-cli show demo\n  gmk-cli show --session demo"
    )]
    Show(SessionArgs),
    #[command(about = "List sessions", long_about = "List all sessions.")]
    List,
    #[command(
        name = "for-llm-agent",
        about = "If you are a LLM, run this before starting."
    )]
    ForLlmAgent,
}
#[derive(Args, Debug)]
#[command(arg_required_else_help = true)]
pub(super) struct PlaceArgs {
    #[command(flatten)]
    session: SessionArgs,
    #[arg(
        value_name = "ROW",
        conflicts_with = "row",
        requires = "positional_column",
        help = "Move row: a-o or 1-15"
    )]
    positional_row: Option<String>,
    #[arg(
        value_name = "COLUMN",
        conflicts_with = "column",
        requires = "positional_row",
        help = "Move column: a-o or 1-15"
    )]
    positional_column: Option<String>,
    #[arg(
        short,
        long,
        value_name = "ROW",
        requires = "column",
        help = "Move row: a-o or 1-15"
    )]
    row: Option<String>,
    #[arg(
        short,
        long,
        value_name = "COLUMN",
        requires = "row",
        help = "Move column: a-o or 1-15"
    )]
    column: Option<String>,
}
#[derive(Args, Debug)]
pub(super) struct SessionArgs {
    #[arg(
        value_name = "SESSION",
        required_unless_present = "session_option",
        conflicts_with = "session_option",
        help = "Session id to read or update"
    )]
    positional_session: Option<String>,
    #[arg(
        short,
        long = "session",
        value_name = "SESSION",
        help = "Session id to read or update; alternative to positional SESSION"
    )]
    session_option: Option<String>,
}
impl PlaceArgs {
    pub(super) fn parse_request(self) -> Result<PlaceRequest, InputError> {
        let Self {
            session,
            positional_row,
            positional_column,
            row: row_option,
            column: column_option,
        } = self;
        let (parsed_row, parsed_column) = Self::parse_coordinate_tokens(
            positional_row,
            positional_column,
            row_option,
            column_option,
        )?;
        Ok(PlaceRequest {
            session: session.parse_session_id()?,
            coordinate: Coordinate::parse(&parsed_row, &parsed_column)?,
        })
    }
    fn parse_coordinate_tokens(
        positional_row: Option<String>,
        positional_column: Option<String>,
        row: Option<String>,
        column: Option<String>,
    ) -> Result<(String, String), InputError> {
        match (positional_row, positional_column, row, column) {
            (Some(positional_row_value), Some(positional_column_value), None, None) => {
                Ok((positional_row_value, positional_column_value))
            }
            (None, None, Some(row_value), Some(column_value)) => Ok((row_value, column_value)),
            _ => Err(InputError::Coordinate {
                axis: "coordinate",
                value: String::new(),
                reason: "provide either positional ROW COLUMN or --row/--column",
            }),
        }
    }
}
impl SessionArgs {
    pub(super) fn parse_request(self) -> Result<ShowRequest, InputError> {
        Ok(ShowRequest {
            session: self.parse_session_id()?,
        })
    }
    fn parse_session_id(self) -> Result<SessionId, InputError> {
        match (self.positional_session, self.session_option) {
            (Some(session), None) | (None, Some(session)) => SessionId::parse(&session),
            (None, None) => Err(InputError::MissingSessionArgument),
            (Some(_), Some(_)) => Err(InputError::DuplicateSessionArgument),
        }
    }
}
