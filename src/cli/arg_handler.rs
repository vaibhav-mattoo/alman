use crate::cli::cli_data::Cli;
use clap::Parser;

pub fn parse_args() -> Cli {
    Cli::parse()
}
