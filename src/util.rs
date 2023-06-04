use std::fs;

use std::path::Path;

use md5::Digest;

pub(crate) fn md5(path: &Path) -> String {
    let content = fs::read(path).unwrap();
    let mut hasher = md5::Md5::default();
    hasher.update(content);
    let hash = hasher.finalize();
    format!("{:x}", hash)
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

#[test]
fn current_dir_list_test() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    // src为相对于项目的根目录
    for entry in walkdir::WalkDir::new(Path::new("src"))
        .follow_links(false)
        .into_iter()
        .filter_map(|f| f.ok())
    {
        log::info!("{:?}", entry);
    }
}

#[test]
fn test_md5() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let file = Path::new("src/main.rs");
    log::info!("{:?}", file);
    let v = md5(file);
    log::info!("{:?}", v);
}
