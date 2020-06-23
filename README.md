# gameboy
DMG-01 Emulator in Rust


## Design Notes

Rough notes to be modified and eventually deleted.

- CPU impl needs to execute instructions.
- each instruction takes a number of cycles and probably wants to return how many cycles it took (emulated of course)

- memory should be its own struct and impl
- this might be what's in control of


- a lot of opcodes are just duplicate but with different targets. For example ADD has a bunch of opcodes for adding A to a differeng register.
- in this case, probably want to implement a single ADD function that accepts source and target.
- Then opcode matching has hard coded in the registers, while also passing relevant decoded arguments from the opcode?



- main loop looks like it should be frame-based, not op based.

- pause at vblank

- run operations until


8KB RAM
8KB VRAM

1mhz (4mhz?)

stack is 16 bit -> Can only push and pop 16 bit registers





Thank Yous

- tobiasvl on Discord Emulation Development was very helpful explaining boot rom behaviour.
