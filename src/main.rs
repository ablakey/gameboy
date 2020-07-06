mod cpu;
mod mmu;
mod opcode;
mod registers;

use cpu::CPU;
use mmu::MMU;

fn main() {
    // TODO: handle ROM when the bootloader is working.
    // let args: Vec<String> = env::args().collect();
    //
    // if args.len() < 2 {
    //     println!("USAGE: {} <rom-file>", args[0]);
    //     return;
    // }

    // let romfile = &args[1];

    let mut emulator = Emulator::new();

    loop {
        emulator.step();
    }
}

struct Emulator {
    // APU
    // PPU  <- Screen
    cpu: CPU,
}

impl Emulator {
    /// Create a new Emulator instance. The only
    fn new() -> Self {
        Self { cpu: CPU::new() }
    }

    fn step(&mut self) {
        self.cpu.step();
    }
}
