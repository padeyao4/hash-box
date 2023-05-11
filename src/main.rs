use std::{fs, io::Write, path::Path};

use zip::{write::FileOptions, ZipWriter};

fn main() {
    let target_path = Path::new("C:\\Users\\11818\\Desktop\\target.zip");

    let source_path = Path::new("C:\\Users\\11818\\Desktop\\source.json");

    let content = std::fs::read_to_string(source_path);

    println!("{}", content.unwrap());

    let target_file = std::fs::File::create(target_path).unwrap();

    println!("{:?}", target_file);

    let mut zip_writer = ZipWriter::new(target_file);

    zip_writer
        .start_file("source.json", FileOptions::default())
        .unwrap();
    zip_writer
        .write(fs::read_to_string(source_path).unwrap().as_bytes())
        .unwrap();
    zip_writer.finish().unwrap();
}

#[test]
fn zip_file_test() {
    let target_path = Path::new("C:\\Users\\11818\\Desktop\\target.zip");

    let source_path = Path::new("C:\\Users\\11818\\Desktop\\source.json");
    let target_file = std::fs::File::create(target_path).unwrap();
    let mut zip_writer = ZipWriter::new(target_file);

    zip_writer
        .start_file("source.json", FileOptions::default())
        .unwrap();
    zip_writer
        .write(fs::read_to_string(source_path).unwrap().as_bytes())
        .unwrap();
    zip_writer.finish().unwrap();
}

#[test]
fn unzip_file_test() {
    use zip::ZipArchive;
    let source_zip_path = Path::new("C:\\Users\\11818\\Desktop\\target.zip");
    let zip_file = fs::File::open(source_zip_path).unwrap();
    let mut zip_archive = ZipArchive::new(zip_file).unwrap();
    let path = Path::new("C:\\Users\\11818\\Desktop\\test");
    zip_archive.extract(path).unwrap();
}
