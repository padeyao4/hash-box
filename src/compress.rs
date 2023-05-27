use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::exit;
use std::{fs, io};

use log::{error, info, warn};

use zip::write::FileOptions;
use zip::{DateTime, ZipArchive, ZipWriter};

/// 压缩目录或者文件，保持md5值不会改变
pub fn zip(src: &Path, dsc: &Path) -> io::Result<()> {
    info!("compress {:?} to {:?}", src, dsc);
    if !Path::exists(src) {
        error!("{} not exists, exit", src.display());
        exit(-1);
    }

    if Path::exists(dsc) {
        warn!("{} exists, rewrite the file", dsc.display());
    }

    let dsc_file = File::create(dsc)?;
    let mut zip = ZipWriter::new(dsc_file);

    let entries = walkdir::WalkDir::new(src)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|f| f.ok());

    let options = FileOptions::default()
        .compression_level(Some(1))
        .last_modified_time(DateTime::default());

    let mut buff = Vec::new();
    for entry in entries {
        let path = entry.path();

        let relation_buf =
            Path::new(src.file_name().unwrap()).join(path.strip_prefix(src).unwrap());
        let relation = relation_buf.as_path();
        let file_path = relation.to_string_lossy();
        let name = file_path.trim_end_matches("/");
        if entry.path_is_symlink() {
            info!("l {}", name);
            zip.add_symlink(name, path.read_link()?.to_string_lossy(), options)?;
        } else if Path::is_dir(entry.path()) {
            info!("d {}", name);
            zip.add_directory(name, options)?;
        } else {
            info!("f {}", name);
            zip.start_file(name, options)?;
            let mut f = File::open(path)?;
            // todo 当读写大文件时,可能会出现内存问题
            f.read_to_end(&mut buff)?;
            zip.write_all(&mut buff)?;
            buff.clear();
        }
    }
    zip.finish()?;
    Ok(())
}

/// unzip file
pub fn unzip(src: &Path, dsc: &Path) -> io::Result<()> {
    if !Path::exists(src) {
        error!("{:?} not exits,will exit!", src);
        exit(-1);
    }
    if Path::exists(dsc) {
        warn!("{:?} exists,will rewrite the files", dsc);
    }
    let f = File::open(src)?;
    let mut archive = ZipArchive::new(f)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let output = match file.enclosed_name() {
            Some(path) => dsc.join(path.to_owned()),
            None => continue,
        };

        {
            let comment = file.comment();
            if !comment.is_empty() {
                info!("file {} comment : {}", i, comment);
            }
        }

        if file.is_file() {
            // todo 考虑软连接问题
            info!(
                "file {} extracted to \"{}\" ({} bytes)",
                i,
                output.display(),
                file.size()
            );
            if let Some(p) = output.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }

            let mut outfile = File::create(&output)?;
            io::copy(&mut file, &mut outfile)?;
        } else {
            info!("directory {} extracted to \"{}\"", i, output.display());
            fs::create_dir_all(&output)?;
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&dsc, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }

    Ok(())
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
fn path_test() {
    use log::debug;
    use std::env::set_var;
    set_var("RUST_LOG", "debug");
    env_logger::init();

    let tmp = tempfile::tempdir().unwrap();
    let tmp_dir = tmp.path();

    let p = tmp_dir.join("hello");
    debug!("{}", p.display());
}
