use crate::cli::{Cli, Commands};
use clap::Parser;

mod cli;
mod constant;
mod model;
mod util;

fn main() {
    use std::env::set_var;
    set_var("RUST_LOG", "debug");
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Add { path, force } => {}
        Commands::Get { name } => {}
        Commands::Delete { name } => {}
        Commands::Sync { path } => {}
        Commands::List { .. } => {}
        Commands::About { .. } => {}
        Commands::Clear { .. } => {}
    }
}

#[test]
fn test_envs() {
    use dirs::home_dir;
    use log::info;
    use std::env;
    use std::path::PathBuf;

    let p = env::var(constant::HBX_HOME_ENV);
    let hbx_path: Option<PathBuf> = match p {
        Ok(p) => Some(p.into()),
        Err(_) => home_dir(),
    };
    println!("{}", hbx_path.unwrap().display());
}
