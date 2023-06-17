mod core;

use crate::core::cli::Commands;
use clap::Parser;
use log::info;

pub const HBX_HOME_ENV: &str = "HBX_HOME";
pub const CONFIG_NAME: &str = "config";
pub const STORE_DIRECTORY: &str = "store";

pub fn run() -> anyhow::Result<()> {
    let mut store = core::store::Store::default()?;
    store.load()?;
    let cli = core::cli::Cli::parse();
    match cli.command {
        Commands::Add { path } => {
            store.add(&path)?;
            store.save()?;
        }
        Commands::Get { name, path } => {
            store.get(&name, path)?;
        }
        Commands::Delete { name } => {
            store.delete(&name);
            store.save()?;
        }
        Commands::List { .. } => {
            let ans = store.list();
            for item in ans {
                println!("{}", item);
            }
        }
        Commands::About { .. } => {
            info!("config {:?}", store.config_path());
            info!("storage {:?}", store.store_dir());
        }
        Commands::Clear { .. } => {
            store.clear()?;
        }
    }
    Ok(())
}
