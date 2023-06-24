mod common;

use crate::common::util::logger_init;
use hbx::core::util::execute;
use log::debug;
use ssh2::Session;

/// 检查ssh是否加载ssh key
/// 在windows下当openssh库太低会出现ssh无法连接
/// 使用winget install Microsoft.OpenSSH.Beta
#[test]
fn test_ssh_session() -> anyhow::Result<()> {
    logger_init()?;
    let sess = Session::new()?;
    let mut agent = sess.agent()?;
    agent.connect()?;
    agent.list_identities()?;
    for identity in agent.identities()? {
        debug!("{:?}", identity.comment());
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_execute() -> anyhow::Result<()> {
    logger_init()?;
    let res = execute("ls /", "root", "127.0.0.1:22")?;
    debug!("{}", res);
    assert!(!res.is_empty());
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_execute_error() -> anyhow::Result<()> {
    logger_init()?;
    let res = execute(
        "command -v java &> /dev/null && echo 0 || echo 1",
        "root",
        "127.0.0.1:22",
    )?;
    debug!("{}", res);
    Ok(())
}
