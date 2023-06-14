use fs::{create_dir_all, hard_link, read_to_string};
use std::collections::HashSet;
use std::fs::read_link;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

use anyhow::{anyhow, bail, Result};
use atomicwrites::{AllowOverwrite, AtomicFile};
use constant::{CONFIG_NAME, STORE_DIRECTORY};
use dirs::home_dir;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};

use crate::constant::HBX_HOME_ENV;
use crate::model::Meta::{DIRECTORY, FILE, SYMLINK};
use crate::util::md5;
use crate::{constant, util};

#[derive(Debug, Deserialize, Serialize)]
pub enum Meta {
    FILE(String),
    SYMLINK(PathBuf),
    DIRECTORY(Vec<Node>),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Node {
    pub name: String,
    pub meta: Meta,
}

impl PartialEq for Node {
    /// 判断节点是否相同，在linux中可以通过inode判断,此处可以优化
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl TryFrom<&Path> for Node {
    type Error = anyhow::Error;

    fn try_from(p: &Path) -> Result<Self, Self::Error> {
        let name = p
            .file_name()
            .ok_or(anyhow!("invalid path"))?
            .to_string_lossy()
            .to_string();

        let meta = if p.is_symlink() {
            SYMLINK(read_link(p)?)
        } else if p.is_dir() {
            DIRECTORY(Vec::new())
        } else {
            FILE(md5(p))
        };

        let n = Self { name, meta };
        Ok(n)
    }
}

impl Node {
    fn sample(s: &str) -> Self {
        Self {
            name: s.to_string(),
            meta: FILE(String::new()),
        }
    }

    fn recursive_link_and_calc(p: &Path, s: &Path) -> Result<Node> {
        let name = p
            .file_name()
            .ok_or(anyhow!("invalidate path"))?
            .to_string_lossy()
            .to_string();

        let meta = if p.is_symlink() {
            SYMLINK(p.read_link()?)
        } else if p.is_dir() {
            let mut children = Vec::new();
            for entry in walkdir::WalkDir::new(p)
                .follow_links(false)
                .sort_by_file_name()
                .max_depth(1)
                .into_iter()
                .filter_map(|f| f.ok())
                .filter(|f| f.path() != p)
            {
                let child = Node::recursive_link_and_calc(entry.path(), s)?;
                children.push(child);
            }
            DIRECTORY(children)
        } else {
            let m = util::md5(&p);
            let dst = s.join(Path::new(&m));
            info!("l {:?} -> {:?}", &p, &dst);
            if !dst.exists() {
                hard_link(&p, &dst)?;
            }
            FILE(m)
        };
        Ok(Node { name, meta })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StoreConfig {
    path: PathBuf,
    data: HashSet<Node>,
}

impl StoreConfig {
    fn new(path: PathBuf) -> Result<Self> {
        create_dir_all(path.join(STORE_DIRECTORY))?;
        let s = Self {
            path,
            data: HashSet::new(),
        };
        Ok(s)
    }

    pub(crate) fn default() -> Result<Self> {
        let p = env::var(HBX_HOME_ENV);
        let hbx_home_path: Option<PathBuf> = match p {
            Ok(p) => Some(p.into()),
            Err(_) => home_dir().map(|f| f.join(PathBuf::from(".hbx"))),
        };

        let path = hbx_home_path.unwrap_or(PathBuf::from("~/.hbx"));
        StoreConfig::new(path)
    }

    pub(crate) fn get(&self, n: &str, path: Option<PathBuf>) -> Result<()> {
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
        ) -> Result<()> {
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

    pub(crate) fn config_path(&self) -> PathBuf {
        self.path.join(Path::new(CONFIG_NAME))
    }

    pub(crate) fn store_dir(&self) -> PathBuf {
        self.path.join(Path::new(STORE_DIRECTORY))
    }

    /// 加载数据
    pub(crate) fn load(&mut self) -> Result<()> {
        let config_path = self.config_path();
        if config_path.exists() {
            let content = read_to_string(&config_path)?;
            let tmp: HashSet<Node> = from_str(&content)?;
            self.data.extend(tmp);
        }
        Ok(())
    }

    pub(crate) fn save(&self) -> Result<()> {
        let s = to_string(&self.data)?;
        AtomicFile::new(self.config_path(), AllowOverwrite).write(|f| f.write_all(s.as_bytes()))?;
        info!("save path is {}", self.config_path().display());
        Ok(())
    }

    pub(crate) fn add(&mut self, path: &Path) -> Result<()> {
        if path.exists() {
            if !self.data.contains(&path.try_into()?) {
                let node = Node::recursive_link_and_calc(path, &self.store_dir())?;
                self.data.insert(node);
            }
        }
        Ok(())
    }

    pub(crate) fn list(&self) -> Vec<&str> {
        let mut ans = Vec::new();
        for x in &self.data {
            ans.push(x.name.as_str());
        }
        ans
    }

    pub(crate) fn delete(&mut self, name: &str) {
        self.data.remove(&Node::sample(name));
    }

    pub(crate) fn clear(&self) -> Result<()> {
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

#[cfg(test)]
mod model_test {
    use std::collections::HashSet;
    use std::fs::{create_dir_all, remove_dir_all};
    use std::path::Path;

    use crate::model::StoreConfig;

    #[test]
    fn test_model() -> anyhow::Result<()> {
        let mut config = StoreConfig::default()?;
        config.load()?;
        config.add(Path::new(".idea"))?;
        config.add(Path::new(".idea"))?;
        config.add(Path::new(".idea"))?;
        assert_eq!(1, config.data.len());
        config.save()?;
        // test loading
        config.load()?;
        config.add(Path::new("src"))?;
        assert_eq!(config.data.len(), 2);
        // test delete
        config.delete("src");
        assert_eq!(config.data.len(), 1);
        remove_dir_all(config.path)?;
        Ok(())
    }

    #[test]
    fn delete_all_dirs_test() -> anyhow::Result<()> {
        let config = StoreConfig::default()?;
        remove_dir_all(config.path)?;
        Ok(())
    }

    #[test]
    fn test_hash_set_extend() {
        let ans = [1, 2, 3];
        let mut set = HashSet::new();
        set.extend(ans);
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn create_dirs() -> anyhow::Result<()> {
        let a = Path::new("target/test");
        create_dir_all(a)?;
        create_dir_all(a)?;
        assert!(a.exists());
        remove_dir_all(a)?;
        assert!(!a.exists());
        Ok(())
    }
}
