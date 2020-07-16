mod emulator;
mod guest;
mod host;
use emulator::Emulator;
use host::init_debugger;

fn main() {
    init_debugger();
    let emulator = Emulator::new();
    emulator.unwrap().run_forever();
}
