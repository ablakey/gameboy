mod emulator;
mod guest;
mod host;
use emulator::Emulator;
use std::env;

fn main() {
    // This is not how you do CLI argument handling. I just didn't want to bother with any
    // additional libraries.
    let args: Vec<String> = env::args().collect();
    let cartridge_path = if args.len() > 1 { Some(&args[1]) } else { None };
    let skip_boot_rom = args.contains(&String::from("--noboot"));

    if skip_boot_rom {
        println!("SKIP BOOT ROM");
    }

    println!("{}", cartridge_path.unwrap());

    let mut emulator = Emulator::new(cartridge_path, !skip_boot_rom).unwrap();
    emulator.run_forever();
}
