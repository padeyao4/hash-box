use clap::Parser;
use log::info;
use std::env;

use dirs::home_dir;
use util::HbxConfig;

use crate::cli::{Cli, Commands};
use crate::util::Handle;

mod cli;
mod util;

fn main() -> std::io::Result<()> {
    use std::env::set_var;
    set_var("RUST_LOG", "debug");
    env_logger::init();

    let hbx_home_opt = env::var(util::HBX_HOME);
    let hbx_home = match hbx_home_opt {
        Ok(p) => Some(p.into()),
        Err(_) => home_dir(),
    };

    let hbx = HbxConfig::new(hbx_home.unwrap());

    let cli = Cli::parse();

    match cli.command {
        Commands::Add { path, force } => {
            hbx.add(&path, force);
        }
        Commands::Get { name } => {
            hbx.get(&name);
        }
        Commands::Delete { name } => {
            hbx.delete(&name);
        }
        Commands::Sync { path } => {
            hbx.sync(&path);
        }
        Commands::List { .. } => {
            hbx.list();
        }
        Commands::About { .. } => {}
        Commands::Clear { .. } => {
            hbx.clear();
        }
    }
    Ok(())
}

#[test]
fn test_envs() {
    use std::path::PathBuf;
    let p = env::var(util::HBX_HOME);
    let hbx_path: Option<PathBuf> = match p {
        Ok(p) => Some(p.into()),
        Err(_) => home_dir(),
    };
    println!("{}", hbx_path.unwrap().display());
}
