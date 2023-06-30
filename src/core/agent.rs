use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

use anyhow::bail;
use log::info;
use ssh2::Session;

pub struct Agent {
    session: Session,
}

impl Agent {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            session: Session::new()?,
        })
    }

    pub fn login(&mut self, username: &str, host: &str) -> anyhow::Result<()> {
        info!("tcp connect...");
        let tcp = TcpStream::connect(host)?;
        self.session.set_tcp_stream(tcp);
        self.session.handshake()?;
        self.session.userauth_agent(username)?;
        if !self.session.authenticated() {
            bail!("authentication failed");
        }
        info!("authenticated success");
        Ok(())
    }

    pub fn download(&self, local_path: &Path, remote_path: &Path) -> anyhow::Result<()> {
        let (mut remote_file, stat) = self.session.scp_recv(remote_path)?;
        info!("remote file size: {}", stat.size());
        let mut contents = Vec::new();
        remote_file.read_to_end(&mut contents)?;

        // Close the channel and wait for the whole content to be transferred
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;
        info!("content: {:?}", &contents);
        fs::write(local_path, contents)?;
        Ok(())
    }

    pub fn upload(&self, local_path: &Path, remote_file: &Path) -> anyhow::Result<()> {
        let size = local_path.metadata()?.len();
        info!("size {}", size);
        let mut channel = self.session.scp_send(remote_file, 0o755, size, None)?;
        info!("scp send");
        channel.write(&fs::read(local_path)?)?;
        info!("write file ok");
        // Close the channel and wait for the whole content to be transferred
        channel.send_eof()?;
        info!("send file ok");
        channel.wait_eof()?;
        info!("wait end of file");
        channel.close()?;
        info!("close file");
        channel.wait_close()?;
        info!("closed ok");
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
