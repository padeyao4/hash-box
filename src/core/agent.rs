use anyhow::bail;
use log::info;
use ssh2::Session;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

pub struct Agent {
    session: Session,
}

impl Agent {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            session: Session::new()?,
        })
    }

    pub fn login(&self, username: &str, address: &str) -> anyhow::Result<()> {
        let tcp = TcpStream::connect(address)?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;
        sess.userauth_agent(username)?;
        sess.agent()?;
        if !sess.authenticated() {
            bail!("authentication failed");
        }
        Ok(())
    }

    pub fn download(&self, local_path: &Path, remote_path: &Path) -> anyhow::Result<()> {
        let (mut remote_file, stat) = self.session.scp_recv(local_path)?;
        info!("remote file size: {}", stat.size());
        let mut contents = Vec::new();
        remote_file.read_to_end(&mut contents)?;

        // Close the channel and wait for the whole content to be transferred
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;

        fs::write(remote_path, contents)?;
        Ok(())
    }

    pub fn upload(&self, local_path: &Path, remote_path: &Path) -> anyhow::Result<()> {
        let size = local_path.metadata()?.len();
        let mut remote_file = self.session.scp_send(remote_path, 0o644, size, None)?;
        remote_file.write(&fs::read(local_path)?)?;
        // Close the channel and wait for the whole content to be transferred
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;
        Ok(())
    }

    pub fn write_remote_file(&self, content: &str, remote_path: &Path) -> anyhow::Result<()> {
        let size = content.len() as u64;
        let mut remote_file = self.session.scp_send(remote_path, 0o644, size, None)?;
        remote_file.write(content.as_bytes())?;
        // Close the channel and wait for the whole content to be transferred
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;
        Ok(())
    }

    pub fn execute(&self, cmd: &str) -> anyhow::Result<String> {
        let mut channel = self.session.channel_session()?;
        channel.exec(cmd)?;
        let mut s = String::new();
        channel.read_to_string(&mut s)?;
        channel.wait_close()?;
        Ok(s)
    }
}
