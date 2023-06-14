use crate::cli::{Cli, Commands};
use crate::model::StoreConfig;
use clap::Parser;
use log::info;

mod cli;
mod constant;
mod model;
mod util;

fn main() -> anyhow::Result<()> {
    use std::env::set_var;
    set_var("RUST_LOG", "debug");
    env_logger::init();

    let mut config = StoreConfig::default()?;
    config.load()?;
    let cli = Cli::parse();
    match cli.command {
        Commands::Add { path } => {
            config.add(&path)?;
            config.save()?;
        }
        Commands::Get { name, path } => {
            config.get(&name, path)?;
        }
        Commands::Delete { name } => {
            config.delete(&name);
            config.save()?;
        }
        Commands::List { .. } => {
            let ans = config.list();
            for item in ans {
                println!("{}", item);
            }
        }
        Commands::About { .. } => {
            info!("config {:?}", config.config_path());
            info!("storage {:?}", config.store_dir());
        }
        Commands::Clear { .. } => {
            config.clear()?;
        }
    }
    Ok(())
}

#[test]
fn test_envs() {
    use dirs::home_dir;
    use std::env;
    use std::path::PathBuf;

    let p = env::var(constant::HBX_HOME_ENV);
    let hbx_path: Option<PathBuf> = match p {
        Ok(p) => Some(p.into()),
        Err(_) => home_dir(),
    };
    println!("{}", hbx_path.unwrap().display());
}
