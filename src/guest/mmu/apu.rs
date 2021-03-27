use super::is_bit_set;

pub struct ApuRegisters {
    // Square (with sweep)
    pub square1_sweep_time: u8,
    pub square1_sweep_increase: bool, // If true, sweep frequency increases. False == decreases.
    pub square1_sweep_shift: u8,
    pub square1_wave_duty: u8,
    pub square1_length: u8,
    pub square1_frequency: u16,
    pub square1_initialize: bool,
    pub square1_length_enabled: bool,
    nr12: u8, // 0xFF12: Sound mode 1 envelope.

    // Square
    pub square2_wave_duty: u8,
    pub square2_length: u8,
    pub square2_frequency: u16,
    pub square2_initialize: bool,
    pub square2_length_enabled: bool,
    nr22: u8, // 0xFF17: Sound mode 2 register, envelope.

    // Wave
    pub wave_on: bool,
    wave_length: u8,
    wave_length_enabled: bool,
    pub wave_output: u8, // 00: mute, 01: as-is, 10: shift right, 11: shift right twice.
    pub wave_frequency: u16, // Two 8-bit registers acting as a frequency value.
    pub wave_ram: [u8; 32], // 32 4-bit wave pattern samples.
    wave_initialize: bool, // When set high, the sound restarts, then flag is set low.

    // Noise
    nr41: u8, // 0xFF20: Sound mode 4 register, length.
    nr42: u8, // 0xFF21: Sound mode 4 register, envelope.
    nr43: u8, // 0xFF22: Sound mode 4 register, polynomial counter.
    nr44: u8, // 0xFF23: Sound mode 4 register, counter/consecutive.
    nr50: u8, // 0xFF24: Channel control, on/off, volume.
    nr51: u8, // 0xFF25: Selection of Sound output terminal.
    nr52: u8, // 0xFF26: Power to sound.
}

impl ApuRegisters {
    pub fn new() -> Self {
        Self {
            square1_sweep_time: 0,
            square1_sweep_increase: false,
            square1_sweep_shift: 0,
            square1_wave_duty: 0,
            square1_length: 0,
            square1_frequency: 0,
            square1_initialize: false,
            square1_length_enabled: false,
            nr12: 0,
            square2_wave_duty: 0,
            square2_length: 0,
            square2_frequency: 0,
            square2_initialize: false,
            square2_length_enabled: false,
            nr22: 0,
            wave_on: true,
            wave_length: 0,
            wave_length_enabled: false,
            wave_output: 0,
            wave_frequency: 0,
            wave_initialize: false,
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
            nr50: 0,
            nr51: 0,
            nr52: 0,
            wave_ram: [0; 32],
        }
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            0xFF10 => {
                self.square1_sweep_time = (value >> 4) & 0x7;
                self.square1_sweep_shift = value & 0x7;
                self.square1_sweep_increase = is_bit_set(value, 3)
            }
            0xFF11 => {
                self.square1_wave_duty = value >> 6; // Highest 2 bits.
                self.square1_length = value & 0x3F; // Lowest 6 bits.
            }
            0xFF12 => self.nr12 = value,
            0xFF13 => {
                self.square1_frequency = (self.square1_frequency & 0xFF00) | (value & 0xFF) as u16
            }
            0xFF14 => {
                // Get the lowest 3 bits from value, shift to bits 9,10,11.
                self.square1_frequency =
                    (self.square1_frequency & 0xFF) | (((value & 0x07) as u16) << 8);
                self.square1_initialize = is_bit_set(value, 7);
                self.square1_length_enabled = is_bit_set(value, 6);
            }
            0xFF16 => {
                self.square2_wave_duty = value >> 6; // Highest 2 bits.
                self.square2_length = value & 0x3F; // Lowest 6 bits.
            }
            0xFF17 => self.nr22 = value,
            0xFF18 => {
                self.square2_frequency = (self.square2_frequency & 0xFF00) | (value & 0xFF) as u16
            }
            0xFF19 => {
                // Get the lowest 3 bits from value, shift to bits 9,10,11.
                self.square2_frequency =
                    (self.square2_frequency & 0xFF) | (((value & 0x07) as u16) << 8);
                self.square2_initialize = is_bit_set(value, 7);
                self.square2_length_enabled = is_bit_set(value, 6);
            }
            0xFF1A => self.wave_on = is_bit_set(value, 7),
            0xFF1B => self.wave_length = value,
            0xFF1C => self.wave_output = (value >> 5) & 0x3, // Only bits 5 and 6 matter.
            0xFF1D => self.wave_frequency = (self.wave_frequency & 0xFF00) | (value & 0xFF) as u16,
            0xFF1E => {
                // Get the lowest 3 bits from value, shift to bits 9,10,11.
                self.wave_frequency = (self.wave_frequency & 0xFF) | (((value & 0x07) as u16) << 8);
                self.wave_initialize = is_bit_set(value, 7);
                self.wave_length_enabled = is_bit_set(value, 6);
            }
            0xFF20 => self.nr41 = value,
            0xFF21 => self.nr42 = value,
            0xFF22 => self.nr43 = value,
            0xFF23 => self.nr44 = value,
            0xFF24 => self.nr50 = value,
            0xFF25 => {
                self.nr51 = value;
                println!("{}", value);
            }
            0xFF26 => self.nr52 = value,
            0xFF30..=0xFF3F => {
                // Incoming 8-bit value is two 4-bit samples. Split it and set it to wave_ram.
                self.wave_ram[(address as usize - 0xFF30) / 2] = value >> 4;
                self.wave_ram[(address as usize - 0xFF30) / 2 + 1] = value & 0xF;
            }
            _ => panic!(
                "Tried to write to an APU register that was not implemented: {:x}",
                address
            ),
        }
    }

    pub fn rb(&self, address: u16) -> u8 {
        println!("{:#}", address);
        0
        // TODO: Implement.
    }

    // pub fn rb(&self, address: u16) -> u8 {
    //     match address {
    //         0xFF14 => self.nr14, // TODO: not correct. Only bit 6 can be read?
    //         0xFF19 => self.nr24,
    //         0xFF1E => self.nr34,
    //         0xFF23 => self.nr44,
    //         _ => panic!(
    //             "Tried to get a hardware register wtih invalid address {:x}",
    //             address
    //         ),
    //     }
    // }
}
