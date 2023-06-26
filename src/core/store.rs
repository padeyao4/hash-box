use crate::core::agent::Agent;
use crate::core::node::Meta::{DIRECTORY, FILE, SYMLINK};
use crate::core::node::Node;
use crate::{CONFIG_NAME, HBX_HOME_ENV, STORE_DIRECTORY};
use anyhow::{anyhow, bail};
use atomicwrites::{AllowOverwrite, AtomicFile};
use dirs::home_dir;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, hard_link, read_to_string};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

#[derive(Debug, Deserialize, Serialize)]
pub struct Store {
    path: PathBuf,
    data: HashSet<Node>,
}

impl Store {
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        create_dir_all(path.join(STORE_DIRECTORY))?;
        let s = Self {
            path,
            data: HashSet::new(),
        };
        Ok(s)
    }

    pub fn default() -> anyhow::Result<Self> {
        let p = env::var(HBX_HOME_ENV);
        let hbx_home_path: Option<PathBuf> = match p {
            Ok(p) => Some(p.into()),
            Err(_) => home_dir().map(|f| f.join(PathBuf::from(".hbx"))),
        };

        let path = hbx_home_path.unwrap_or(PathBuf::from("~/.hbx"));
        Store::new(path)
    }

    pub fn get(&self, name: &str, dst: Option<PathBuf>) -> anyhow::Result<()> {
        let dst = dst.unwrap_or(PathBuf::from("./"));
        if !dst.exists() {
            bail!("{:?} not exits! exit", dst);
        }
        if dst.is_file() {
            bail!("{:?} is a file, please input a directory path", dst)
        }
        let root = match self.data.get(&Node::sample(name)) {
            None => {
                bail!("{} not exists, exit!", name);
            }
            Some(n) => n,
        };
        self.recover(root, &dst.join(&root.name))?;
        Ok(())
    }

    // 恢复数据
    fn recover(&self, node: &Node, dst: &Path) -> anyhow::Result<()> {
        match &node.meta {
            FILE(value) => {
                let src = self.store_dir().join(Path::new(&value));
                info!("l {:?} -> {:?}", &src, &dst);
                hard_link(src, dst)?;
            }
            SYMLINK(path) => {
                std::os::unix::fs::symlink(path, dst)?;
            }
            DIRECTORY(vec) => {
                info!("d {:?}", dst);
                fs::create_dir(&dst)?;
                for x in vec.borrow().iter() {
                    self.recover(x, &dst.join(Path::new(&x.name)))?;
                }
            }
        }
        Ok(())
    }

    pub fn config_path(&self) -> PathBuf {
        self.path.join(Path::new(CONFIG_NAME))
    }

    pub fn store_dir(&self) -> PathBuf {
        self.path.join(Path::new(STORE_DIRECTORY))
    }

    /// 加载数据
    pub fn load(&mut self) -> anyhow::Result<()> {
        let config_path = self.config_path();
        if config_path.exists() {
            let content = read_to_string(&config_path)?;
            let tmp: HashSet<Node> = from_str(&content)?;
            self.data.extend(tmp);
        } else {
            fs::write(config_path, "[]")?;
        }
        Ok(())
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let s = to_string(&self.data)?;
        AtomicFile::new(self.config_path(), AllowOverwrite).write(|f| f.write_all(s.as_bytes()))?;
        info!("save path is {}", self.config_path().display());
        Ok(())
    }

    pub fn add(&mut self, path: &Path) -> anyhow::Result<()> {
        if path.exists() {
            if !self.data.contains(&Node::try_from(path)?) {
                let root = self.build(path)?;
                self.links(&root, path)?;
                self.data.insert(root);
            }
        }
        Ok(())
    }

    fn build(&self, path: &Path) -> anyhow::Result<Node> {
        info!("build {:?}", path);
        let root = Node::new(path)?;
        for entry in walkdir::WalkDir::new(path)
            .follow_links(false)
            .sort_by_file_name()
            .max_depth(1)
            .into_iter()
            .filter_map(|f| f.ok())
            .filter(|f| f.path() != path)
        {
            let node = if entry.path().is_dir() {
                self.build(entry.path())?
            } else {
                Node::new(entry.path())?
            };

            match &root.meta {
                DIRECTORY(vec) => {
                    vec.borrow_mut().push(node);
                }
                _ => {}
            }
        }
        Ok(root)
    }

    fn links(&self, root: &Node, src: &Path) -> anyhow::Result<()> {
        match &root.meta {
            FILE(value) => {
                let dst = self.store_dir().join(Path::new(value));
                info!("l {:?} -> {:?}", &src, &dst);
                hard_link(src, dst)?;
            }
            SYMLINK(_) => {}
            DIRECTORY(vec) => {
                for node in vec.borrow().iter() {
                    self.links(node, &src.join(Path::new(&node.name)))?;
                }
            }
        }
        Ok(())
    }

    pub fn list(&self) -> Vec<&str> {
        let mut ans = Vec::new();
        for x in &self.data {
            ans.push(x.name.as_str());
        }
        ans
    }

    pub fn delete(&mut self, name: &str) {
        self.data.remove(&Node::sample(name));
    }

    pub fn clear(&self) -> anyhow::Result<()> {
        let names = walkdir::WalkDir::new(self.store_dir())
            .follow_links(false)
            .into_iter()
            .filter_map(|f| f.ok())
            .filter(|p| p.path() != self.store_dir())
            .map(|p| p.file_name().to_string_lossy().to_string())
            .collect::<HashSet<String>>();
        let mut tmp = HashSet::new();

        fn dfs(node: &Node, tmp: &mut HashSet<String>) {
            match &node.meta {
                FILE(x) => {
                    tmp.insert(x.to_owned());
                }
                DIRECTORY(nodes) => {
                    for x in nodes.borrow().iter() {
                        dfs(x, tmp);
                    }
                }
                _ => {}
            };
        }

        for node in &self.data {
            dfs(&node, &mut tmp);
        }

        let res: HashSet<_> = names
            .difference(&tmp)
            .map(|name| self.store_dir().join(PathBuf::from(name)))
            .collect();

        for path in res {
            info!("delete {:?}", path);
            fs::remove_file(path)?;
        }

        Ok(())
    }

    pub fn info(&self) -> anyhow::Result<String> {
        let mut map = HashMap::<String, String>::new();
        map.insert(
            "config".into(),
            self.config_path().to_string_lossy().to_string(),
        );
        map.insert(
            "storage".into(),
            self.store_dir().to_string_lossy().to_string(),
        );
        Ok(to_string(&map)?)
    }

    pub fn pull(
        &mut self,
        names: Vec<String>,
        address: String,
        port: Option<i32>,
    ) -> anyhow::Result<()> {
        info!("pull tools {:?} from {:?}", names, address);
        let address: Vec<&str> = address.split('@').collect();

        let username = address[0];
        let net_address = address[1].to_string() + ":" + port.unwrap_or(22).to_string().as_str();

        // 登陆远程服务器
        let mut agent = Agent::new()?;
        agent.login(username, &net_address)?;

        // 判断服务上是否安装hbx命令
        let res = agent.execute("[ -f /usr/local/bin/hbx ] && echo 0 || echo 1")?;

        // 如果服务器上安装没安装hbx
        if res.eq("1") {
            bail!("server not install hbx, exiting")
        }

        // 读取服务器端配置信息
        let info = agent.execute("/usr/local/bin/hbx info")?;
        let map: HashMap<String, String> = from_str(&info)?;

        // 下载配置文件到本地
        let remote_config = map.get("config").ok_or(anyhow!("config info error"))?;
        let tmp = tempfile::tempdir()?;
        let dst_dir = tmp.path();
        agent.download(dst_dir, &PathBuf::from(remote_config))?;

        // 加载远程配置文件
        let content = read_to_string(&dst_dir.join(CONFIG_NAME))?;
        let remote_data: HashSet<Node> = from_str(&content)?;

        // 比对差异文件
        let s1 = Store::get_files(&self.data);
        let s2 = Store::get_files(&remote_data);
        let s3 = s2.difference(&s1).collect::<HashSet<&String>>();

        // 下载差异文件
        let remote_storage = map.get("storage").ok_or(anyhow!("storage info error"))?;
        let local_storage = self.store_dir();
        for item in s3 {
            let remote = PathBuf::from(remote_storage).join(PathBuf::from(item));
            let local = local_storage.join(PathBuf::from(item));
            info!("download {:?} to {:?}", &remote, &local);
            agent.download(&local, &remote)?;
        }

        // 合并远程和本地配置
        self.data.extend(remote_data);
        Ok(())
    }

    fn get_files(data: &HashSet<Node>) -> HashSet<String> {
        let mut ans = HashSet::new();
        for item in data.iter() {
            if let FILE(s) = &item.meta {
                ans.insert(s.to_string());
            }
        }
        ans
    }

    pub fn push(
        &self,
        names: Vec<String>,
        address: String,
        port: Option<String>,
    ) -> anyhow::Result<()> {
        info!("{:?} {}", names, address);
        let address: Vec<&str> = address.split("@").collect();

        let username = address[0];
        let host = address[1].to_string() + ":" + port.unwrap_or("22".into()).as_str();
        info!("username : {}, host: {}", username, host);

        // 登陆远程服务器
        let mut agent = Agent::new()?;
        info!("login");
        agent.login(username, &host)?;

        info!("check hbx if exists");
        // 判断服务上是否安装hbx命令
        let res = agent.execute("[ -f /usr/local/bin/hbx ] && echo ok || echo fail")?;

        // 如果服务器上安装没安装hbx,上传hbx命令到服务器
        if res.trim().eq("fail") {
            info!("server not install hbx, upload hbx to server");
            let bin_path = PathBuf::from("/usr/local/bin/hbx");
            agent.upload(&bin_path, &bin_path)?;
        }

        // 读取服务器端配置信息
        let info = agent.execute("/usr/local/bin/hbx info")?;
        let map: HashMap<String, String> = from_str(&info)?;

        // 下载配置文件到本地
        let remote_config = map.get("config").ok_or(anyhow!("config info error"))?;
        let tmp = tempfile::tempdir()?;
        let dst_dir = tmp.path();
        info!("download config from server");
        agent.download(dst_dir, &PathBuf::from(remote_config))?;

        // 加载远程配置文件
        let content = read_to_string(&dst_dir.join(CONFIG_NAME))?;
        let mut remote_data: HashSet<Node> = from_str(&content)?;

        // 比对差异文件
        let local_data = Store::get_files(&self.data);
        let s2 = Store::get_files(&remote_data);
        let s3 = local_data.difference(&s2).collect::<HashSet<&String>>();

        // 上传差异文件
        let remote_storage = map.get("storage").ok_or(anyhow!("storage info error"))?;
        let local_storage = self.store_dir();
        for item in s3 {
            let remote = PathBuf::from(remote_storage).join(PathBuf::from(item));
            let local = local_storage.join(PathBuf::from(item));
            info!("upload {:?} to {:?}", &remote, &local);
            agent.upload(&local, &remote)?;
        }

        // 合并本地配置到远程
        remote_data.extend(self.data.clone());
        let s = to_string(&remote_data)?;
        agent.write_remote_file(&s, &PathBuf::from(remote_config))?;
        Ok(())
    }
}
