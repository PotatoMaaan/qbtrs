#![allow(clippy::needless_return)]
#![allow(clippy::needless_borrow)]

use clap::Parser;
use cli::BaseCommand;
use cli_handler::handle_cli;
use config::{get_dirs, Config};

mod backend;
mod cli;
mod cli_handler;
mod config;

fn main() {
    let args = BaseCommand::parse();
    let dirs = get_dirs();
    let mut config = Config::from_file(&dirs);

    handle_cli(args, &dirs, &mut config);

    config.save_config(&dirs);
}
