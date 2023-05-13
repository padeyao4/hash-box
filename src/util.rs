use std::fs;
use std::io::Write;
use std::path::Path;
use log::log;
use zip::write::FileOptions;
use zip::ZipWriter;

pub fn zip(source: &Path, target: &Path) -> std::io::Result<()> {
    // if Path::exists(target) {
    //     log!("hell","1212");
    // }

    // let target_file = fs::File::create(target_path).unwrap();
    // let mut zip_writer = ZipWriter::new(target_file);
    //
    // zip_writer
    //     .start_file("source.json", FileOptions::default())
    //     .unwrap();
    // zip_writer
    //     .write(fs::read_to_string(source_path).unwrap().as_bytes())
    //     .unwrap();
    // zip_writer.finish().unwrap();
    Ok(())
}

pub fn unzip(source: &Path, target: &Path) {}

pub fn sum_md5(path: &Path) {}