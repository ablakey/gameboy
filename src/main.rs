mod emulator;
mod guest;
mod host;
use emulator::Emulator;
use std::env;
mod debug;
use std::panic;

fn main() {
    let args: Vec<String> = env::args().collect();

    let cartridge_path = if args.len() > 1 { Some(&args[1]) } else { None };

    println!("{}", cartridge_path.unwrap());

    let mut emulator = Emulator::new(cartridge_path).unwrap();

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| emulator.run_forever()));

    match result {
        Err(e) => {
            emulator.dump_state();
            panic!("{:?}", e);
        }
        _ => (),
    }
}
