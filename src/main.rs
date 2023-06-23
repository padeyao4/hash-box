use log::info;
use std::env::set_var;
use std::io::Write;
use std::process;

use hbx::run;

fn main() {
    set_var("RUST_LOG", "DEBUG");
    let mut builder = env_logger::Builder::new();
    builder.format(|buf, record| writeln!(buf, "[ {} ] {}", buf.timestamp(), record.args()));
    builder.parse_default_env();
    builder.init();

    if let Err(e) = run() {
        info!("An error occurred at runtime {}", e);
        process::exit(1);
    }
}
