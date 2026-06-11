use gmk_cli::cli;
use std::process::ExitCode;
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
fn main() -> ExitCode {
    cli::run()
}
