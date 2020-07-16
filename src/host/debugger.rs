use std::fs::File;

use simplelog::{Config, LevelFilter, WriteLogger};

pub fn init_debugger() {
    WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("/tmp/gameboy.log").unwrap(),
    )
    .unwrap();
}
