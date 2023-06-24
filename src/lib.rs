pub mod core;

use crate::core::cli::Commands;
use clap::Parser;

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
        Commands::Info { .. } => {
            println!("{}", store.info()?);
        }
        Commands::Clear { .. } => {
            store.clear()?;
        }
        Commands::Pull {
            names,
            address,
            port,
        } => {
            store.pull(names, address, port)?;
        }
        Commands::Push {
            names,
            address,
            port,
        } => {
            store.push(names, address, port)?;
        }
    }
    Ok(())
}
