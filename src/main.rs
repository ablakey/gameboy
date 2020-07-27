mod emulator;
mod guest;
mod host;
use emulator::Emulator;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("USAGE: {} <rom-file>", args[0]);
        return;
    }

    let cartridge_path = &args[1];

    let emulator = Emulator::new(Some(cartridge_path));
    emulator.unwrap().run_forever();
}
