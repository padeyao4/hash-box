use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, hard_link, read_to_string};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

use anyhow::{anyhow, bail};
use atomicwrites::{AllowOverwrite, AtomicFile};
use dirs::home_dir;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};

use crate::core::agent::Agent;
use crate::core::node::Meta::{DIRECTORY, FILE, SYMLINK};
use crate::core::node::Node;
use crate::{CONFIG_NAME, HBX_HOME_ENV, STORE_DIRECTORY};

#[derive(Debug, Deserialize, Serialize)]
pub struct Store {
    path: PathBuf,
    data: HashSet<Node>,
}

impl Store {
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        create_dir_all(path.join(STORE_DIRECTORY))?;
        let config_path = path.join(CONFIG_NAME);
        if !config_path.exists() {
            fs::write(config_path, "[]")?;
        }
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

    fn save(&self) -> anyhow::Result<()> {
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
                self.save()?;
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

    pub fn delete(&mut self, name: &str) -> anyhow::Result<()> {
        self.data.remove(&Node::sample(name));
        self.save()?;
        self.clear()?;
        Ok(())
    }

    fn clear(&self) -> anyhow::Result<()> {
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
        address: String,
        names: Vec<String>,
        port: Option<String>,
        all: bool,
    ) -> anyhow::Result<()> {
        let agent = Self::login_server(address, port)?;

        if !Self::remote_has_hbx(&agent)? {
            bail!("server not install hbx");
        }

        // 读取服务器端配置信息
        let map = Self::remote_hbx_info(&agent)?;
        let remote_config = map.get("config").ok_or(anyhow!("config info error"))?;
        let remote_storage = map.get("storage").ok_or(anyhow!("storage info error"))?;

        // 下载配置文件到本地
        let tmp = tempfile::tempdir()?;
        let dst_file = tmp.path().join(CONFIG_NAME);
        agent.download(&dst_file, &PathBuf::from(remote_config))?;

        // 加载远程配置文件
        let remote_data: HashSet<Node> = from_str(&read_to_string(&dst_file)?)?;

        // 比对差异文件
        let mut target = HashSet::new();
        Self::filter(names, all, &remote_data, &mut target)?;
        let diff = Self::get_diff(
            &mut target,
            &mut self.data.iter().collect::<HashSet<&Node>>(),
        )?;

        // 下载差异文件
        for item in diff {
            let remote = PathBuf::from(remote_storage).join(PathBuf::from(&item));
            let local = self.store_dir().join(PathBuf::from(&item));
            if !&local.exists() {
                agent.download(&local, &remote)?;
            }
        }

        // 合并远程和本地配置
        self.data.extend(target.into_iter().map(|f| f.to_owned()));

        self.save()?;
        Ok(())
    }

    fn remote_has_hbx(agent: &Agent) -> anyhow::Result<bool> {
        // 判断服务上是否安装hbx命令
        let res = agent.execute("[ -f /usr/local/bin/hbx ] && echo 0 || echo 1")?;
        let res = res.trim();
        Ok(res.eq("0"))
    }

    fn get_files(data: &mut dyn Iterator<Item = &Node>) -> HashSet<String> {
        let mut ans = HashSet::new();
        for item in data {
            if let FILE(s) = &item.meta {
                ans.insert(s.to_string());
            }
            if let DIRECTORY(children) = &item.meta {
                ans.extend(Store::get_files(&mut children.borrow().iter()));
            }
        }
        ans
    }

    pub fn push(
        &self,
        address: String,
        names: Vec<String>,
        port: Option<String>,
        install: bool,
        all: bool,
    ) -> anyhow::Result<()> {
        let agent = Self::login_server(address, port)?;

        if !Self::remote_has_hbx(&agent)? {
            if install {
                info!("server install hbx ...");
                agent.upload(&env::current_exe()?, &PathBuf::from("/usr/local/bin/hbx"))?;
            } else {
                bail!("remote server not install hbx!!!");
            }
        }

        // 读取服务器端配置信息
        let map = Self::remote_hbx_info(&agent)?;
        // 下载配置文件到本地
        let remote_config = map.get("config").ok_or(anyhow!("config info error"))?;
        let remote_storage = map.get("storage").ok_or(anyhow!("storage info error"))?;

        let tmp = tempfile::tempdir()?;
        let dst_file = tmp.path().join(CONFIG_NAME);
        agent.download(&dst_file, &PathBuf::from(remote_config))?;

        // 加载远程配置文件
        let mut remote_data: HashSet<Node> = from_str(&read_to_string(&dst_file)?)?;

        // 计算差异
        let mut target = HashSet::new();
        Self::filter(names, all, &self.data, &mut target)?;
        let diff = Self::get_diff(&mut target, &mut remote_data.iter().collect())?;

        // 上传差异文件
        for item in diff {
            let remote = PathBuf::from(remote_storage).join(PathBuf::from(&item));
            let local = self.store_dir().join(PathBuf::from(&item));
            agent.upload(&local, &remote)?;
        }

        // 合并本地配置到远程
        remote_data.extend(target.into_iter().map(|f| f.to_owned()));
        agent.write_remote_file(&to_string(&remote_data)?, &PathBuf::from(remote_config))?;
        Ok(())
    }

    fn get_diff(src: &HashSet<&Node>, other: &HashSet<&Node>) -> anyhow::Result<HashSet<String>> {
        let ans = Self::get_files(&mut src.iter().map(|f| f.to_owned()))
            .difference(&Self::get_files(&mut other.iter().map(|f| f.to_owned())))
            .map(|s| s.to_string())
            .collect();
        Ok(ans)
    }

    fn filter<'a>(
        names: Vec<String>,
        all: bool,
        set: &'a HashSet<Node>,
        ans: &mut HashSet<&'a Node>,
    ) -> anyhow::Result<()> {
        if all {
            ans.extend(set);
        } else {
            for name in names {
                ans.insert(
                    set.get(&Node::sample(&name))
                        .ok_or(anyhow!("not contain the name"))?,
                );
            }
        }
        Ok(())
    }

    fn remote_hbx_info(agent: &Agent) -> anyhow::Result<HashMap<String, String>> {
        let info = agent.execute("hbx info")?;
        let info = info.trim();
        info!("remote info: {}", info);
        let map = from_str::<HashMap<String, String>>(&info)?;
        Ok(map)
    }

    fn login_server(address: String, port: Option<String>) -> anyhow::Result<Agent> {
        // 正则判断是别名，还是网络地址 todo
        let address: Vec<&str> = address.split("@").collect();

        let username = address[0];
        let host = address[1].to_string() + ":" + port.unwrap_or("22".into()).as_str();
        info!("username : {}, host: {}", username, host);

        // 登陆远程服务器
        let mut agent = Agent::new()?;
        agent.login(username, &host)?;
        Ok(agent)
    }
}
