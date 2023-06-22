use anyhow::bail;
use log::info;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

use md5::Digest;
use ssh2::Session;

pub fn md5(path: &Path) -> anyhow::Result<String> {
    let content = fs::read(path)?;
    let mut hasher = md5::Md5::default();
    hasher.update(content);
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

/// 从服务器上下载文件到本地
pub fn download(
    username: &str,
    address: &str,
    local_path: &Path,
    remote_path: &Path,
) -> anyhow::Result<()> {
    let sess = ssh_session(username, address)?;

    let (mut remote_file, stat) = sess.scp_recv(local_path)?;
    info!("remote file size: {}", stat.size());
    let mut contents = Vec::new();
    remote_file.read_to_end(&mut contents)?;

    // Close the channel and wait for the whole content to be tranferred
    remote_file.send_eof()?;
    remote_file.wait_eof()?;
    remote_file.close()?;
    remote_file.wait_close()?;

    fs::write(remote_path, contents)?;
    Ok(())
}

/// 上传本地文件到服务端
pub fn upload(
    username: &str,
    address: &str,
    local_path: &Path,
    remote_path: &Path,
) -> anyhow::Result<()> {
    // Connect to the local SSH server
    let tcp = TcpStream::connect(address)?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    sess.userauth_agent(username)?;

    let size = local_path.metadata()?.len();
    let mut remote_file = sess.scp_send(remote_path, 0o644, size, None)?;
    remote_file.write(&fs::read(local_path)?)?;
    // Close the channel and wait for the whole content to be transferred
    remote_file.send_eof()?;
    remote_file.wait_eof()?;
    remote_file.close()?;
    remote_file.wait_close()?;
    Ok(())
}

/// 获取ssh session
fn ssh_session(username: &str, address: &str) -> anyhow::Result<Session> {
    let tcp = TcpStream::connect(address)?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    sess.userauth_agent(username)?;
    sess.agent()?;
    if !sess.authenticated() {
        bail!("authentication failed");
    }
    Ok(sess)
}

/// 执行远程服务器上命令
pub fn execute(cmd: &str, username: &str, address: &str) -> anyhow::Result<String> {
    let sess = ssh_session(username, address)?;
    let mut channel = sess.channel_session()?;
    channel.exec(cmd)?;
    let mut s = String::new();
    channel.read_to_string(&mut s)?;
    channel.wait_close()?;
    Ok(s)
}
