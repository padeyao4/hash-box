use crate::core::node::Meta::{DIRECTORY, FILE, SYMLINK};
use crate::core::util::md5;
use anyhow::anyhow;
use log::info;
use serde::{Deserialize, Serialize};
use std::fs::{hard_link, read_link};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

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
            DIRECTORY(Vec::new())
        } else {
            FILE(md5(p))
        };

        let n = Self { name, meta };
        Ok(n)
    }
}

impl Node {
    pub(crate) fn sample(s: &str) -> Self {
        Self {
            name: s.to_string(),
            meta: FILE(String::new()),
        }
    }

    pub(crate) fn recursive_link_and_calc(p: &Path, s: &Path) -> anyhow::Result<Node> {
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
            let m = md5(&p);
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
