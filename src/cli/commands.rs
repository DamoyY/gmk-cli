use crate::coordinate::{Coordinate, parse_axis};
use crate::errors::InputError;
use crate::session_id::SessionId;
use clap::{Args, Parser, Subcommand, ValueEnum};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaceRequest {
    pub session: SessionId,
    pub coordinate: Coordinate,
    pub output_format: OutputFormat,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShowRequest {
    pub session: SessionId,
    pub output_format: OutputFormat,
}
#[expect(
    clippy::exhaustive_enums,
    reason = "supported output formats are closed"
)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Lm,
    Human,
    Json,
    Blindfold,
}
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Play Gomoku with a bot in the terminal.\n\nX: Black\nY: White\n\nYour opponent may be slower, please be patient.",
    arg_required_else_help = true,
    subcommand_required = true,
    after_help = "Examples:\n  gmk-cli place --session demo 8 h\n  gmk-cli show --session demo\n  gmk-cli list"
)]
pub struct Cli {
    #[command(subcommand)]
    pub(super) command: Command,
}
#[derive(Subcommand, Debug)]
pub(super) enum Command {
    #[command(
        about = "Place stone on an existing or new board.",
        after_help = "Examples:\n  gmk-cli place --session demo 1 a\n  gmk-cli place --session demo h 8"
    )]
    Place(PlaceArgs),
    #[command(
        about = "Show the board.",
        long_about = "Show the current board for a session without placing a stone.",
        after_help = "Examples:\n  gmk-cli show --session demo\n  gmk-cli show --session demo --output-format human"
    )]
    Show(ShowArgs),
    #[command(about = "List sessions", long_about = "List all sessions.")]
    List,
    #[command(name = "i-admit-defeat", about = "Admit defeat immediately.")]
    IAdmitDefeat(SessionArgs),
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
    #[command(flatten)]
    output: OutputArgs,
    #[arg(
        value_name = "COORDINATE",
        num_args = 2,
        help = "Move coordinates: one numeric row from 1 to 15 and one letter column from a to o, in either order"
    )]
    coordinates: Vec<String>,
}
#[derive(Args, Debug)]
pub(super) struct ShowArgs {
    #[command(flatten)]
    session: SessionArgs,
    #[command(flatten)]
    output: OutputArgs,
}
#[derive(Args, Debug)]
pub(super) struct SessionArgs {
    #[arg(
        short,
        long = "session",
        required = true,
        value_name = "SESSION",
        help = "Session id to read or update"
    )]
    session: String,
}
#[derive(Args, Debug)]
struct OutputArgs {
    #[arg(long = "output-format", value_enum, default_value = "lm")]
    format: OutputFormat,
}
impl PlaceArgs {
    pub(super) fn parse_request(self) -> Result<PlaceRequest, InputError> {
        let Self {
            session,
            output,
            coordinates,
        } = self;
        let coordinate = Self::parse_coordinate_tokens(&coordinates)?;
        Ok(PlaceRequest {
            session: session.parse_session_id()?,
            coordinate,
            output_format: output.format,
        })
    }
    fn parse_coordinate_tokens(coordinates: &[String]) -> Result<Coordinate, InputError> {
        if coordinates.len() != 2 {
            return Err(InputError::Coordinate {
                axis: "coordinate",
                value: coordinates.join(" "),
                reason: "provide exactly one numeric row and one letter column",
            });
        }
        let mut row = None;
        let mut column = None;
        for coordinate in coordinates {
            Self::parse_coordinate_token(&mut row, &mut column, coordinate)?;
        }
        let row_index = Self::required_coordinate_axis(row, "row")?;
        let column_index = Self::required_coordinate_axis(column, "column")?;
        Coordinate::new(row_index, column_index)
    }
    fn parse_coordinate_token(
        row: &mut Option<usize>,
        column: &mut Option<usize>,
        token: &str,
    ) -> Result<(), InputError> {
        if token.is_empty() {
            return Err(InputError::Coordinate {
                axis: "coordinate",
                value: token.to_owned(),
                reason: "expected a row number from 1 to 15 or a column label from a to o",
            });
        }
        if token.bytes().all(|byte| byte.is_ascii_digit()) {
            let row_index = parse_axis("row", token)?;
            return Self::set_coordinate_axis(row, "row", token, row_index);
        }
        if token.bytes().all(|byte| byte.is_ascii_alphabetic()) {
            let column_index = parse_axis("column", token)?;
            return Self::set_coordinate_axis(column, "column", token, column_index);
        }
        Err(InputError::Coordinate {
            axis: "coordinate",
            value: token.to_owned(),
            reason: "expected a row number from 1 to 15 or a column label from a to o",
        })
    }
    fn set_coordinate_axis(
        target: &mut Option<usize>,
        axis: &'static str,
        token: &str,
        index: usize,
    ) -> Result<(), InputError> {
        if target.is_some() {
            return Err(InputError::Coordinate {
                axis,
                value: token.to_owned(),
                reason: "coordinate must contain exactly one row and one column",
            });
        }
        *target = Some(index);
        Ok(())
    }
    fn required_coordinate_axis(
        value: Option<usize>,
        axis: &'static str,
    ) -> Result<usize, InputError> {
        value.ok_or_else(|| InputError::Coordinate {
            axis,
            value: String::new(),
            reason: "coordinate must contain exactly one row and one column",
        })
    }
}
impl ShowArgs {
    pub(super) fn parse_request(self) -> Result<ShowRequest, InputError> {
        Ok(ShowRequest {
            session: self.session.parse_session_id()?,
            output_format: self.output.format,
        })
    }
}
impl SessionArgs {
    pub(super) fn parse_session_id(self) -> Result<SessionId, InputError> {
        SessionId::parse(&self.session)
    }
}
