use crate::args::Cli;
use clap::Parser;

mod args;
mod cargo;
mod curl;
mod error;
mod git;
mod manager;
mod npm;
mod utils;

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    manager::handle_cli_args(cli);
}
