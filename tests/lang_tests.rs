mod common;

use crate::common::util::logger_init;
use log::debug;

#[test]
fn test_char() -> anyhow::Result<()> {
    logger_init()?;
    let address = "root@127.0.0.1";
    let arr: Vec<&str> = address.split('@').collect();
    debug!("{:?}", arr[0]);
    debug!("{:?}", arr[1]);
    Ok(())
}
