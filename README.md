# Game Boy Emulator
DMG-01 Emulator in Rust

This exists as a way to learn how to write an emulator and get some practice with Rust.

I intentionally endeavour to use the simplest Rust features possible.


## Usage

1. Get a ROM. Don't ask me where to find them.

2. `cargo run myrom.gb --noboot`

## Controls

Keyboard arrows, A, S, Z, X.

## Boot Loader

There is a fully functional boot loader `if` you have `dmg_rom.bin` located in the `data` directory. If not, then you must use `--noboot` to skip running the bootloader and explicitly set all memory, flags, registers to the state that the boot loader would have set them to. Many games depend on assuming this state at initialization.
