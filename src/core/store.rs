use crate::core::node::Meta::{DIRECTORY, FILE, SYMLINK};
use crate::core::node::Node;
use crate::{CONFIG_NAME, HBX_HOME_ENV, STORE_DIRECTORY};
use anyhow::bail;
use atomicwrites::{AllowOverwrite, AtomicFile};
use dirs::home_dir;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::collections::HashSet;
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
}

impl Store {
    pub fn get(&self, n: &str, path: Option<PathBuf>) -> anyhow::Result<()> {
        let p = path.unwrap_or(PathBuf::from("./"));
        if p.is_file() {
            bail!("{:?} is a file, please input a directory path", p)
        }
        let root = match self.data.get(&Node::sample(n)) {
            None => bail!("not contain the value {}", n),
            Some(node) => node,
        };

        fn dfs(
            node: &Node,
            parent: &Path,
            store: &Path,
            tmp: &mut Vec<(PathBuf, PathBuf)>,
        ) -> anyhow::Result<()> {
            let name = &node.name;
            let dst = parent.join(PathBuf::from(name));

            match &node.meta {
                FILE(s) => {
                    if !&dst.exists() {
                        let src = store.join(PathBuf::from(s));
                        info!("f {:?}", &dst);
                        hard_link(src, &dst)?;
                    }
                }
                SYMLINK(link) => {
                    #[cfg(windows)]
                    {
                        if link.exists() {
                            info!("l {:?} -> {:?}", dst, link);
                            if link.is_dir() {
                                std::os::windows::fs::symlink_dir(dst, link)?;
                            } else {
                                std::os::windows::fs::symlink_file(dst, link)?;
                            }
                        } else {
                            tmp.push((dst.into(), link.into()));
                        }
                    }
                    #[cfg(linux)]
                    {
                        std::os::unix::fs::symlink(dst, l)?;
                    }
                }
                DIRECTORY(children) => {
                    info!("d {:?}", dst);
                    fs::create_dir(&dst)?;
                    for x in children {
                        dfs(x, &dst, &store, tmp)?;
                    }
                }
            }
            Ok(())
        }
        let base = &self.store_dir();
        let mut tmp = Vec::<(PathBuf, PathBuf)>::new();
        dfs(root, &p, base, &mut tmp)?;
        for (src, dst) in tmp {
            info!("l {:?} -> {:?}", src, dst);
            if dst.is_dir() {
                std::os::windows::fs::symlink_dir(src, dst)?;
            } else {
                std::os::windows::fs::symlink_file(src, dst)?;
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
            if !self.data.contains(&path.try_into()?) {
                let node = Node::recursive_link_and_calc(path, &self.store_dir())?;
                self.data.insert(node);
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
                    for x in nodes {
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
}

impl Store {
    pub fn pull(&self, names: Vec<String>, address: String) -> anyhow::Result<()> {
        info!("pull tools {:?} from {:?}", names, address);
        // todo: implement
        Ok(())
    }
}
