use log::debug;
use std::env::set_var;
use std::fs;
use std::path::PathBuf;

pub fn test_env_init() -> anyhow::Result<()> {
    set_var("RUST_LOG", "DEBUG");
    env_logger::try_init()?;
    let test_env_path = PathBuf::from(".env_test");
    debug!("{:?}", &fs::canonicalize(&test_env_path));
    dotenv::from_path(test_env_path)?;
    Ok(())
}
