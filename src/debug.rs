use pretty_hex;
use std::fs::{create_dir, File};
use std::io::prelude::*;

pub fn format_hex(data: &Vec<u8>, start_index: u16) -> String {
    data.chunks(16)
        .enumerate()
        .map(|(n, c)| {
            format!(
                "{:04x}: {}\n",
                n * 16 + start_index as usize,
                pretty_hex::simple_hex(&c.to_vec())
            )
        })
        .collect()
}

/// TODO:
/// Accept a 1024 bytes and make 32 lines of 32 bytes each.
pub fn format_tilemap(data: &[u8]) -> String {
    data.chunks(32)
        .map(|n| {
            let foo: String = n.iter().map(|i| format!("{:02x} ", i)).collect();
            format!("{}\n", foo)
        })
        .collect()
}

pub fn dump_to_file(contents: String, filename: &str) {
    create_dir("/tmp/gameboy").ok();
    let mut file = File::create(format!("/tmp/gameboy/{}", filename)).unwrap();
    write!(file, "{}", contents);
}
