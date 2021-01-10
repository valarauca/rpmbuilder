#![allow(dead_code, unused_imports)]

#[macro_use]
extern crate serde;
extern crate chrono;
extern crate clap;
extern crate errors;
extern crate libflate;
extern crate rpm;
extern crate toml;

mod changelog;
mod cli;
mod core;
mod fileopts;
mod rpm_meta;
mod scripts;
mod sign;
mod versions;
use self::cli::{cli_build, AppWork};

fn main() {
    use std::process::exit;

    let args = cli_build();
    let work = AppWork::from_args(&args);
    match work.work() {
        Ok(()) => exit(0),
        Err(e) => {
            eprintln!("{}", e);
            exit(1)
        }
    }
}
