use std::path::{Path, PathBuf};

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

pub trait Handle {
    fn add(&self, path: &Path, d: bool);

    fn delete(&self, name: &str);

    fn get(&self, name: &str);

    fn list(&self) -> Vec<String>;

    fn sync(&self, path: &Path) -> bool;

    fn clear(&self);

    fn about(&self) -> String;
}

enum T {
    SYMLINK(PathBuf),
    FILE(String),
    DIRECTORY,
}

struct Node {
    t: T,
    name: String,
    children: Vec<Node>,
}

impl Handle for HbxConfig {
    fn add(&self, path: &Path, force: bool) {
        for entry in walkdir::WalkDir::new(path)
            .follow_links(false)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|e| e.ok())
        {
            println!("{:?}", entry);
            // todo handle file
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
    let p = std::env::current_exe().unwrap();
    println!("{}", p.display());
    let entries = walkdir::WalkDir::new(&p)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.path() != &p);
    for entry in entries {
        println!("{:?}", entry);
    }
}
