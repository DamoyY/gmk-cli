#![expect(
    clippy::exhaustive_enums,
    reason = "this binary crate exposes closed protocol and error enums for tests"
)]
#![expect(
    clippy::missing_inline_in_public_items,
    reason = "the public API is an IO-oriented CLI and test surface, not a hot generic library"
)]
#![expect(
    clippy::module_name_repetitions,
    reason = "names such as RoomId and SessionStore are clearer at call sites"
)]
pub mod board;
pub mod cli;
pub mod coordinate;
pub mod errors;
pub mod lock;
pub mod persistence;
pub mod room;
pub mod session;
pub mod stone;
