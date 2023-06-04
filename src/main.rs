use clap::Parser;
use log::info;
use std::env;

use dirs::home_dir;

use crate::cli::{Cli, Commands};

mod cli;
mod constant;
mod model;
mod util;

fn main() -> std::io::Result<()> {
    use std::env::set_var;
    set_var("RUST_LOG", "debug");
    env_logger::init();

    let hbx_home_opt = env::var(constant::HBX_HOME_ENV);
    let hbx_home = match hbx_home_opt {
        Ok(p) => Some(p.into()),
        Err(_) => home_dir(),
    };

    // let hbx = HbxConfig::new(hbx_home.unwrap());

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
    Ok(())
}

#[test]
fn test_envs() {
    use std::path::PathBuf;
    let p = env::var(constant::HBX_HOME_ENV);
    let hbx_path: Option<PathBuf> = match p {
        Ok(p) => Some(p.into()),
        Err(_) => home_dir(),
    };
    println!("{}", hbx_path.unwrap().display());
}
