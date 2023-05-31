use clap::Parser;
use log::info;
use std::env;

use config::HbxConfig;
use dirs::home_dir;

use crate::cli::{Cli, Commands};
use crate::config::Handle;

mod cli;
mod config;

fn main() -> std::io::Result<()> {
    use std::env::set_var;
    set_var("RUST_LOG", "debug");
    env_logger::init();

    let hbx_path_result = env::var(config::HBX_PATH);
    let hbx_path = match hbx_path_result {
        Ok(p) => Some(p.into()),
        Err(_) => home_dir(),
    };

    let hbx_config = HbxConfig::new(hbx_path.unwrap());

    let cli = Cli::parse();

    match cli.command {
        Commands::Add { path } => {
            hbx_config.add(&path);
        }
        Commands::Get { name } => {
            hbx_config.get(&name);
        }
        Commands::Delete { name } => {
            hbx_config.delete(&name);
        }
        Commands::Sync { path } => {
            hbx_config.sync(&path);
        }
        Commands::List { .. } => {
            hbx_config.list();
        }
        Commands::About { .. } => {
            info!("version {}", config::VERSION);
        }
        Commands::Clear { .. } => {
            hbx_config.clear();
        }
    }
    Ok(())
}

#[test]
fn test_envs() {
    let p = env::var("HBX_PATH");
    let hbx_path: Option<PathBuf> = match p {
        Ok(p) => Some(p.into()),
        Err(_) => home_dir(),
    };
    println!("{}", hbx_path.unwrap().display());
}
