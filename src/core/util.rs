use std::fs;
use std::path::Path;

use md5::Digest;

pub fn md5(path: &Path) -> anyhow::Result<String> {
    let content = fs::read(path)?;
    let mut hasher = md5::Md5::default();
    hasher.update(content);
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}
