use std::path::PathBuf;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    /// input directory path
    #[arg(short, long)]
    pub(crate) input: PathBuf,
    ///output directory path
    #[arg(short, long)]
    pub(crate) output: PathBuf,
    /// template directory path
    #[arg(short, long)]
    pub(crate) temp: PathBuf,
}