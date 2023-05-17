use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Zip directory or file to destination
    Zip {
        src: PathBuf,
        dsc: PathBuf,
    },

    /// Unzip file
    Unzip {
        src: PathBuf,
        dsc: PathBuf,
    },

    /// Store files, the current file is divided into small files, compressed and stored
    Store {},

    /// Incrementally synchronize files from the server side
    Sync {},
}
