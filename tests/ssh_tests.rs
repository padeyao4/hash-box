mod common;

use log::debug;
use ssh2::Session;

/// 检查ssh是否加载ssh key
/// 在windows下当openssh库太低会出现ssh无法连接
/// 使用winget install Microsoft.OpenSSH.Beta
#[test]
fn test_ssh_session() -> anyhow::Result<()> {
    common::util::set_log()?;
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
    set_log()?;
    let res = execute("ls /", "root", "127.0.0.1:22")?;
    debug!("{}", res);
    assert!(!res.is_empty());
    Ok(())
}
