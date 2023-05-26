use std::env::set_var;

use clap::Parser;

use crate::cli::{Cli, Commands};

mod cli;
mod compress;

fn main() -> std::io::Result<()> {
    set_var("RUST_LOG", "debug");
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Zip { src, dsc } => {
            compress::zip(src.as_path(), dsc.as_path())?;
        }
        Commands::Unzip { src, dsc } => {
            compress::unzip(src.as_path(), dsc.as_path())?;
        }
        Commands::Store { .. } => {
            todo!("store")
        }
        Commands::Sync { .. } => {
            todo!("sync file")
        }
    }
    Ok(())
}
