use std::env::set_var;

pub fn logger_init() -> anyhow::Result<()> {
    set_var("RUST_LOG", "DEBUG");
    env_logger::try_init()?;
    Ok(())
}
