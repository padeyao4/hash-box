use std::{fs, io::Write, path::Path};
use std::env::{set_var, var};

use clap::Parser;
use log::{info, log};
use tempfile::{tempdir, tempfile};
use zip::{write::FileOptions, ZipWriter};

use crate::cli::{Cli, Commands};

mod util;
mod cli;

fn main() -> std::io::Result<()> {
    set_var("RUST_LOG", "debug");
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Zip { src, dsc } => {
            util::zip(src.as_path(), dsc.as_path())?;
        }
        Commands::Unzip { src, dsc } => {
            util::unzip(src.as_path(), dsc.as_path())?;
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