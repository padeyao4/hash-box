use std::{fs, io, path};
use std::fs::File;
use std::io::{Error, Read, stderr, Write};
use std::path::Path;
use std::process::exit;

use log::{error, info, warn};
use zip::{DateTime, ZipArchive, ZipWriter};
use zip::read::ZipFile;
use zip::write::FileOptions;

/// 压缩目录或者文件，保持md5值不会改变
pub fn zip(src: &Path, dsc: &Path) -> io::Result<()> {
    info!("compress {:?} to {:?}",src,dsc);
    if !Path::exists(src) {
        error!("{} not exists, exit",src.display());
        exit(-1);
    }

    if Path::exists(dsc) {
        warn!("{} exists, rewrite the file",dsc.display());
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
        .unix_permissions(0o755)
        .last_modified_time(DateTime::default());

    let mut buff = Vec::new();
    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy();
        if entry.path_is_symlink() {
            info!("l {:?}",path);
            zip.add_symlink(file_name, path.read_link()?.to_string_lossy(), options)?;
        } else if Path::is_dir(entry.path()) {
            info!("d {:?}",path);
            zip.add_directory(file_name, options)?;
        } else {
            info!("f {:?}",path);
            zip.start_file(file_name, options)?;
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
        error!("{:?} not exits,will exit!",src);
        exit(-1);
    }
    if Path::exists(dsc) {
        warn!("{:?} exists,will rewrite the files",dsc);
    }
    let f = File::open(src)?;
    let mut archive = ZipArchive::new(f)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let output = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue
        };

        {
            let comment = file.comment();
            if !comment.is_empty() {
                info!("file {i} comment : {comment}");
            }
        }

        // todo 解压需要考虑软连接
        if file.name().ends_with('/') {
            info!("file {} extracted to \"{}\"",i,output.display());
            fs::create_dir_all(&output)?;
        } else {
            info!("file {} extracted to \"{}\" ({} bytes)",i,output.display(),file.size());
            if let Some(p) = output.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&output)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }

    Ok(())
}

/// sum the file of md5 value
pub fn md5(path: &Path) {
    todo!("sum md5 value");
}