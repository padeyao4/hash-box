use std::{fs, io::Write, path::Path};
use std::env::{set_var, var};
use clap::Parser;

use log::{info, log};
use tempfile::{tempdir, tempfile};
use zip::{write::FileOptions, ZipWriter};
use crate::cli::Cli;

mod util;
mod cli;

fn main() -> std::io::Result<()> {
    set_var("RUST_LOG", "debug");
    env_logger::init();
    let command = Cli::parse();

    let input_path = command.input;
    info!("input path {:?}",input_path);
    let output_path = command.output;
    info!("output path {:?}",output_path);
    let template_path = command.temp;
    info!("template path {:?}",template_path);

    util::zip(input_path.as_path(), output_path.as_path())?;
    Ok(())
}

#[test]
fn zip_file_test() {
    let target_path = Path::new("C:\\Users\\11818\\Desktop\\target.zip");

    let source_path = Path::new("C:\\Users\\11818\\Desktop\\source.json");
    let target_file = fs::File::create(target_path).unwrap();
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

#[test]
fn test_temp_dir() {
    {
        let temp = tempfile().unwrap();
        println!("{:?}", temp);
    }
}

#[test]
fn log_test() {
    set_var("RUST_LOG", "debug");
    env_logger::init();
    info!("hello");
}

#[test]
fn zip_test() {
    set_var("RUST_LOG", "debug");
    env_logger::init();
    // util::zip(Path::new("Cargo.lock"), Path::new("1.zip"),tempfile()?.).unwrap();
    let mut temp_dir = tempdir();
    let a = &temp_dir.unwrap();
    let p = a.path();
    info!("temp dir {:?}",p);
}