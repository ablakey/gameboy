mod gamepak;

use gamepak::GamePak;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("USAGE: {} <rom-file>", args[0]);
        return;
    }

    let filename = &args[1];

    let emulator = Emulator::new(filename);

    emulator.gamepak.print_debug();
}

struct Emulator {
    // APU
    // PPU  <- Screen
    // CPU
    // Memory <- GamePak
    gamepak: GamePak, // Input
}

impl Emulator {
    fn new(path: &String) -> Self {
        Self {
            gamepak: GamePak::load_from_rom(path).unwrap(),
        }
    }
}
