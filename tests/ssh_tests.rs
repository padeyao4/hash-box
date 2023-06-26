mod common;

use crate::common::util::test_env_init;
use hbx::core::agent::Agent;
use hbx::core::util::execute;
use log::debug;
use ssh2::Session;

/// 检查ssh是否加载ssh key
/// 在windows下当openssh库太低会出现ssh无法连接
/// 使用winget install Microsoft.OpenSSH.Beta
#[test]
fn test_ssh_session() -> anyhow::Result<()> {
    test_env_init()?;
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
    test_env_init()?;
    let res = execute(
        "ls /",
        &std::env::var("TEST_USER")?,
        &std::env::var("TEST_HOST")?,
    )?;
    debug!("{}", res);
    assert!(!res.is_empty());
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_execute_error() -> anyhow::Result<()> {
    test_env_init()?;
    let res = execute(
        "command -v java &> /dev/null && echo 0 || echo 1",
        "root",
        "127.0.0.1:22",
    )?;
    debug!("{}", res);
    Ok(())
}

#[test]
fn test_ssh_login() -> anyhow::Result<()> {
    test_env_init()?;
    let mut agent = Agent::new()?;
    let username = &std::env::var("TEST_USER")?;
    let host = &std::env::var("TEST_HOST")?;
    debug!("username: {}, host: {}", username, host);
    agent.login(username, host)?;
    let res = agent.execute("[ -f /usr/local/hbx ] && echo 0 || echo 1")?;
    debug!("{}", res);
    Ok(())
}
