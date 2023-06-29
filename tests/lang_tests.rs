mod common;

use crate::common::util::test_env_init;
use log::debug;

#[test]
fn test_char() -> anyhow::Result<()> {
    test_env_init()?;
    let address = "root@127.0.0.1";
    let arr: Vec<&str> = address.split('@').collect();
    debug!("{:?}", arr[0]);
    debug!("{:?}", arr[1]);
    Ok(())
}
#[test]
fn test_str_eq() -> anyhow::Result<()> {
    test_env_init()?;
    let s1 = String::from("hello");
    let s2 = "hello";
    assert!(s1.eq(s2));
    let s3 = "hello";
    assert!(s2.eq(s3));
    Ok(())
}
