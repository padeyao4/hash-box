use crate::util::T::{DIRECTORY, FILE, SYMLINK};
use dirs::home_dir;
use md5::Digest;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env::VarError;
use std::error::Error;
use std::fs::{File, FileType};
use std::hash::{Hash, Hasher};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::{env, fs};
use walkdir::DirEntry;

pub const HBX_HOME: &str = "HBX_HOME";
pub const DEFAULT_HBX_PATH: &str = "";

const TEMPLATE_PREFIX: &str = ".hbx_";
const CONFIG_NAME: &str = "config";
const STORE_DIRECTORY: &str = "store";
const TEMPLATE_DIR: &str = "tmp";

#[derive(Debug)]
pub struct HbxConfig {
    path: PathBuf,
}

impl HbxConfig {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// 读取默认配置
    pub fn default() -> Self {
        let p = env::var(HBX_HOME);
        let hbx_path: Option<PathBuf> = match p {
            Ok(p) => Some(p.into()),
            Err(_) => home_dir(),
        };
        Self {
            path: hbx_path.unwrap(),
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn config_path(&self) -> PathBuf {
        self.path.join(CONFIG_NAME)
    }

    pub fn store_path(&self) -> PathBuf {
        self.path.join(STORE_DIRECTORY)
    }

    pub fn template_path(&self) -> PathBuf {
        self.path.join(TEMPLATE_DIR)
    }
}

fn md5(path: &Path) -> String {
    let content = fs::read(path).unwrap();
    let mut hasher = md5::Md5::default();
    hasher.update(content);
    let hash = hasher.finalize();
    format!("{:x}", hash)
}

#[derive(Debug, Deserialize, Serialize)]
enum T {
    SYMLINK(PathBuf),
    FILE(String),
    DIRECTORY,
}

impl T {
    pub fn new(path: &Path) -> T {
        if path.is_symlink() {
            return SYMLINK(path.read_link().unwrap());
        }
        if path.is_dir() {
            return DIRECTORY;
        }
        return FILE(md5(path));
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Node {
    t: T,
    name: String,
    children: Vec<Node>,
}

impl Node {
    pub fn new(path: &Path) -> Node {
        let mut t;
        if path.is_symlink() {
            t = SYMLINK(path.read_link().unwrap());
        } else if path.is_dir() {
            t = DIRECTORY;
        } else {
            t = FILE(md5(path));
        }

        let name = path.file_name().unwrap().to_string_lossy().to_string();

        Node {
            t,
            name,
            children: Vec::new(),
        }
    }
}

impl PartialEq<Self> for Node {
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

pub struct HbxUtil {
    config: HbxConfig,
    data: HashSet<Node>,
}

impl HbxUtil {
    pub fn new(config: HbxConfig) -> Self {
        HbxUtil {
            config,
            data: HashSet::new(),
        }
    }

    pub fn add(&mut self, p0: Node) {
        self.data.insert(p0);
    }

    pub fn read(&mut self) {
        let config_path = self.config.config_path();
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

    pub fn save(&self) {
        use atomicwrites::{AtomicFile, DisallowOverwrite};

        let af = AtomicFile::new(self.config.config_path(), DisallowOverwrite);
        let s = serde_json::to_string(&self.data).unwrap();
        af.write(|f| f.write_all(s.as_bytes()))
            .expect_err("save data failed,some error occur");
    }
}

struct Service {
    local_config: Option<HbxConfig>,
    remote_config: Option<HbxConfig>,
    temp_config: Option<HbxConfig>,
}

impl Service {
    fn new() -> Self {
        Service {
            local_config: Some(HbxConfig::default()),
            remote_config: None,
            temp_config: None,
        }
    }

    fn add(&self, path: &Path, force: bool) {
        let mut hbx = HbxUtil::new(self.local_config.unwrap());
        hbx.read();

        let root = Node::new(path);
        hbx.add(root);

        // todo 遍历目录
        for entry in walkdir::WalkDir::new(path)
            .follow_links(false)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|f| f.path() != path)
        {
            let ty = entry.file_type();
            if ty.is_symlink() {
                // symlink
            } else if ty.is_dir() {
                // directory
            } else {
                // file
            }
        }
    }

    fn delete(&self, name: &str) {
        todo!()
    }

    fn get(&self, name: &str) {
        todo!()
    }

    fn list(&self) -> Vec<String> {
        todo!()
    }

    fn sync(&self, path: &Path) -> bool {
        todo!()
    }

    fn clear(&self) {
        todo!()
    }

    fn about(&self) -> String {
        todo!()
    }
}

#[test]
fn walk_dir_test() {
    use log::debug;
    use std::env::set_var;
    set_var("RUST_LOG", "debug");
    env_logger::init();
    let tmp = tempfile::tempdir().unwrap();
    let tmp_dir = tmp.path();
    debug!("tmp dir {:?}", tmp_dir);
    let entries = walkdir::WalkDir::new(tmp_dir)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|f| f.ok());
    for entry in entries {
        debug!("entry {:?}", entry);
    }
}

#[test]
fn walk_file_test() {
    let p = env::current_exe().unwrap();
    println!("{}", p.display());
    let entries = walkdir::WalkDir::new(&p)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.path() != &p);
    for entry in entries {
        println!("{:?}", entry);
    }
}
