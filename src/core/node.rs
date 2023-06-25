use crate::core::node::Meta::{DIRECTORY, FILE, SYMLINK};
use crate::core::util::md5;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs::read_link;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Meta {
    FILE(String),
    SYMLINK(PathBuf),
    DIRECTORY(Rc<RefCell<Vec<Node>>>),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Node {
    pub name: String,
    pub meta: Meta,
}

impl PartialEq<Self> for Node {
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
            DIRECTORY(Rc::new(RefCell::new(Vec::new())))
        } else {
            FILE(md5(p)?)
        };

        let n = Self { name, meta };
        Ok(n)
    }
}

impl Node {
    pub fn sample(s: &str) -> Self {
        Self {
            name: s.to_string(),
            meta: FILE(String::new()),
        }
    }

    pub fn new(p: &Path) -> anyhow::Result<Node> {
        let name = p
            .file_name()
            .ok_or(anyhow!("invalidate path"))?
            .to_string_lossy()
            .to_string();
        let meta = if p.is_symlink() {
            SYMLINK(p.read_link()?)
        } else if p.is_dir() {
            DIRECTORY(Rc::new(RefCell::new(Vec::new())))
        } else {
            FILE(md5(&p)?)
        };
        Ok(Node { name, meta })
    }
}
