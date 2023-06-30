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
    Add {
        /// the path of the file
        path: PathBuf,
        // #[arg(short, long)]
        // force: bool,
    },

    Delete {
        /// package name
        name: String,
    },

    Get {
        /// name
        name: String,
        /// the path to save files
        path: Option<PathBuf>,
    },

    List {},

    Info {},

    Clear {},

    Pull {
        /// ip or host , eg. root@127.0.0.1
        address: String,
        /// package name ,split by ' '
        names: Vec<String>,
        /// server port
        #[arg(short)]
        port: Option<i32>,
    },

    Push {
        /// ip or host. eg. root@127.0.0.1
        address: String,
        /// package name ,split by ' '
        names: Vec<String>,
        /// server port
        #[arg(short)]
        port: Option<String>,
        /// if server not install hbx then install hbx
        #[arg(short)]
        force: bool,
    },
    Test {},
}
