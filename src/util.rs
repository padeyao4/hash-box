use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use log::{error, info, warn};
use zip::write::FileOptions;
use zip::ZipWriter;

pub fn zip(src: &Path, dsc: &Path) -> std::io::Result<()> {
    if !Path::exists(src) {
        error!("{} not exists, exit",src.display());
        exit(-1);
    }

    if Path::exists(dsc) {
        warn!("{} exists, rewrite the file",dsc.display());
    }

    let dsc_file = File::create(dsc)?;
    let mut zip_writer = ZipWriter::new(dsc_file);

    let entries = walkdir::WalkDir::new(src)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|f| f.ok());

    for entry in entries {
        // todo 根据不同文件类型处理
        if entry.path_is_symlink() {
            info!("symlink {:?}",entry);
        } else if Path::is_dir(entry.path()) {
            info!("directory {:?}",entry);
        } else {
            info!("file {:?}",entry);
        }
    }

    // zip_writer.start_file("hello", FileOptions::default())?;
    // zip_writer.write_all("hello world".as_bytes())?;
    // zip_writer.finish()?;
    Ok(())
}

pub fn unzip(source: &Path, target: &Path) {}

pub fn sum_md5(path: &Path) {}