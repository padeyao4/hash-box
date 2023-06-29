use std::env::set_var;

pub fn test_env_init() -> anyhow::Result<()> {
    set_var("RUST_LOG", "DEBUG");
    env_logger::try_init()?;
    Ok(())
}
