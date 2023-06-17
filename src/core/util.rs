use md5::Digest;
use std::fs;
use std::path::Path;

pub fn md5(path: &Path) -> String {
    let content = fs::read(path).unwrap();
    let mut hasher = md5::Md5::default();
    hasher.update(content);
    let hash = hasher.finalize();
    format!("{:x}", hash)
}
