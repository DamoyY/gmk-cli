use super::ascii_escape;
use core::fmt;
#[expect(clippy::exhaustive_enums, reason = "closed input validation failures")]
#[expect(clippy::module_name_repetitions, reason = "clearer at call sites")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InputError {
    EmptySession,
    MissingSessionArgument,
    DuplicateSessionArgument,
    SessionTooLong {
        max: usize,
    },
    NonAsciiSession,
    UnsafeSessionFileName,
    ReservedSessionFileName,
    UnknownSession {
        session: String,
    },
    Coordinate {
        axis: &'static str,
        value: String,
        reason: &'static str,
    },
}
impl fmt::Display for InputError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[expect(clippy::pattern_type_mismatch, reason = "borrowed match avoids moves")]
        match self {
            Self::EmptySession => write!(f, "session id must not be empty"),
            Self::MissingSessionArgument => write!(f, "session argument is required"),
            Self::DuplicateSessionArgument => write!(f, "session argument was provided twice"),
            Self::SessionTooLong { max } => {
                write!(
                    f,
                    "session id must contain at most {max} printable ASCII bytes"
                )
            }
            Self::NonAsciiSession => {
                write!(f, "session id must contain printable ASCII bytes only")
            }
            Self::UnsafeSessionFileName => write!(
                f,
                "session id must be usable as a SQLite file name and must not contain '<', '>', ':', '\"', '/', '\\', '|', '?', or '*'",
            ),
            Self::ReservedSessionFileName => {
                write!(f, "session id must not be a reserved Windows device name")
            }
            Self::UnknownSession { session } => {
                write!(f, "session '{}' does not exist", ascii_escape(session))
            }
            Self::Coordinate {
                axis,
                value,
                reason,
            } => write!(
                f,
                "{axis} must be valid: {reason}; got '{}'",
                ascii_escape(value),
            ),
        }
    }
}
