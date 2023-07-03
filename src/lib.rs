use clap::Parser;

use crate::core::cli::Commands;

pub mod core;

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
        }
        Commands::Get { name, path } => {
            store.get(&name, path)?;
        }
        Commands::Delete { name } => {
            store.delete(&name)?;
        }
        Commands::List { .. } => {
            for item in store.list() {
                println!("{}", item);
            }
        }
        Commands::Info { .. } => {
            println!("{}", store.info()?);
        }
        Commands::Pull {
            address,
            names,
            port,
            all,
        } => {
            store.pull(address, names, port, all)?;
        }
        Commands::Push {
            address,
            names,
            port,
            install,
            all,
        } => {
            store.push(address, names, port, install, all)?;
        }
    }
    Ok(())
}
