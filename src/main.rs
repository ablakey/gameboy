mod emulator;
mod guest;
mod host;
use emulator::Emulator;
use std::env;
mod debug;
use std::panic;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("USAGE: {} <rom-file>", args[0]);
        return;
    }

    let cartridge_path = &args[1];

    let mut emulator = Emulator::new(Some(cartridge_path)).unwrap();

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| emulator.run_forever()));

    match result {
        Err(e) => {
            emulator.dump_state();
            panic!("{:?}", e);
        }
        _ => (),
    }
}
