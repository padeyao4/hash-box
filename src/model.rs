use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{env, fs};

use dirs::home_dir;
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::constant::HBX_HOME_ENV;
use crate::model::Meta::FILE;
use crate::{constant, util};

#[derive(Debug, Deserialize, Serialize)]
enum Meta {
    FILE(String),
    SYMLINK(PathBuf),
    DIRECTORY(Vec<Node>),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Node {
    name: String,
    meta: Meta,
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
        // 每个元素都有hash方法吗
        self.name.hash(state);
    }
}

impl From<&Path> for Node {
    fn from(value: &Path) -> Self {
        let name = value.file_name().unwrap().to_string_lossy().to_string();

        let meta;
        if value.is_symlink() {
            meta = Meta::SYMLINK(value.read_link().unwrap());
        } else if value.is_dir() {
            let mut children = Vec::new();
            for entry in walkdir::WalkDir::new(value)
                .follow_links(false)
                .sort_by_file_name()
                .max_depth(1)
                .into_iter()
                .filter_map(|f| f.ok())
                .filter(|f| f.path() != value)
            {
                let child = Node::from(entry.path());
                children.push(child);
            }
            meta = Meta::DIRECTORY(children);
        } else {
            meta = FILE(util::md5(value));
        }
        Node { name, meta }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StoreConfig {
    path: PathBuf,
    data: HashSet<Node>,
}

impl StoreConfig {
    fn new(path: PathBuf) -> StoreConfig {
        Self {
            path,
            data: HashSet::new(),
        }
    }

    fn default() -> StoreConfig {
        let p = env::var(HBX_HOME_ENV);
        let hbx_home_path: Option<PathBuf> = match p {
            Ok(p) => Some(p.into()),
            Err(_) => home_dir().map(|f| f.join(PathBuf::from(".hbx"))),
        };
        Self {
            path: hbx_home_path.unwrap(),
            data: HashSet::new(),
        }
    }

    fn config_path(&self) -> PathBuf {
        self.path.join(Path::new(constant::CONFIG_NAME))
    }

    fn store_path(&self) -> PathBuf {
        self.path.join(Path::new(constant::STORE_DIRECTORY))
    }

    /// 加载数据
    fn load(&mut self) {
        let config_path = self.config_path();
        if !config_path.exists() {
            return;
        }
        let content = fs::read_to_string(config_path)
            .expect("Couldn't read config file,maybe it's not a valid config");
        let tmp: Vec<Node> = serde_json::from_str(&content).expect("it's not a valid json file");
        for n in tmp.into_iter() {
            self.data.insert(n);
        }
    }

    fn save(&self) {
        let config_path = self.config_path();

        if !config_path.exists() {
            fs::create_dir_all(config_path.parent().unwrap()).expect("Couldn't create config")
        }

        use atomicwrites::{AllowOverwrite, AtomicFile};
        let af = AtomicFile::new(self.config_path(), AllowOverwrite);
        let s = serde_json::to_string(&self.data).unwrap();
        af.write(|f| f.write_all(s.as_bytes()))
            .expect("save data failed,some error occur");
        info!("save path is {}", self.config_path().display());
    }

    fn add(&mut self, path: &Path) {
        if !path.exists() {
            error!("path not exists, existing");
            exit(1);
        }

        let node = Node::from(path);
        self.data.insert(node);
    }

    fn list(&self) -> Vec<&str> {
        let mut ans = Vec::new();
        for x in &self.data {
            ans.push(x.name.as_str());
        }
        ans
    }

    fn delete(&mut self, name: &str) {
        self.data.remove(&Node {
            name: name.to_owned(),
            meta: FILE("".to_string()),
        });
    }
}

#[test]
fn test_model() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let mut config = StoreConfig::default();
    let path = config.config_path();
    fs::remove_file(path).unwrap();
    config.load();
    config.add(Path::new(".idea"));
    config.add(Path::new(".idea"));
    config.add(Path::new(".idea"));
    config.save();
    // test loading
    config.load();
    config.add(Path::new("src"));
    info!("size {}", config.data.len());
    assert_eq!(config.data.len(), 2);
    // test list
    let lst = config.list();
    info!("{:?}", lst);
    // test delete
    config.delete("src");
    let lst = config.list();
    info!("{:?}", lst);
    assert_eq!(config.data.len(), 1);
}
