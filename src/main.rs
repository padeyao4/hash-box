use log::info;
use std::process;

use hbx::run;

fn main() {
    use std::env::set_var;
    set_var("RUST_LOG", "INFO");
    env_logger::init();
    if let Err(e) = run() {
        info!("An error occurred at runtime {}", e);
        process::exit(1);
    }
}
