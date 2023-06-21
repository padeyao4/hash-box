use std::env::set_var;

pub(crate) fn set_log() -> anyhow::Result<()> {
    set_var("RUST_LOG", "DEBUG");
    env_logger::try_init()?;
    Ok(())
}
