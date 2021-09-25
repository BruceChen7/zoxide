mod app;
mod config;
mod db;
mod error;
mod fzf;
mod shell;
mod util;

use crate::app::{App, Run};
use crate::error::SilentExit;

use clap::Clap;

use std::env;
use std::io::{self, Write};
use std::process;

pub fn main() {
    // Forcibly disable backtraces.
    env::remove_var("RUST_LIB_BACKTRACE");
    env::remove_var("RUST_BACKTRACE");

    // 执行run的trait
    // 这个是程序的入口
    // 根据不同选选项运行不同的部分
    // parse()返回的是Enum App
    if let Err(e) = App::parse().run() {
        match e.downcast::<SilentExit>() {
            Ok(SilentExit { code }) => process::exit(code),
            Err(e) => {
                let _ = writeln!(io::stderr(), "zoxide: {:?}", e);
                process::exit(1);
            }
        }
    }
}
