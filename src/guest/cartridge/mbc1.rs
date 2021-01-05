use super::Mbc;

pub struct Mbc1 {
    data: Vec<u8>,
    ram: [u8; 0x2000],
    rom_bank_number: u8, // A 5-bit register that selects which ROM bank (0x01-0x1F)
}

impl Mbc1 {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            ram: [0; 0x2000], // TODO: this can actually be up to 4 banks (32KB).
            rom_bank_number: 0x01,
        }
    }
}

impl Mbc for Mbc1 {
    /// Read 0x0000 - 0x3FFF directly. Read 0x4000 - 0x7FFF from the currently active memory bank.
    fn rb(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.data[address as usize],
            0x4000..=0x7FFF => {
                // Offset the ROM bank addressing based on which bank is active.
                // For example, if ROM bank 2 is selected (the third 16KB), the offset is 32KB.
                // The address begins at 0x4000 so we subtract 1 bank.  Bank 0 cannot be accessed
                // from here.

                let offset = 0x4000 * self.rom_bank_number as usize;
                self.data[(address as usize - 0x4000) + offset]
            }
            0xA000..=0xBFFF => {
                println!("Read RAM");
                self.ram[(address - 0xA000) as usize]
            }
            _ => {
                panic!("Tried to read from {:#x} which is not mapped.", address);
            }
        }
    }

    fn wb(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => panic!("Tried to write to RAM enable bit."),
            0x2000..=0x3FFF => {
                let bank = value & 0x1F; // Mask out top 3 bits.
                self.rom_bank_number = bank;
            }
            0xA000..=0xBFFF => {
                self.ram[(address - 0xA000) as usize] = value;
            }
            _ => panic!(
                "Unsupported write to MBC1. Address {:#x}. Value {:#x}",
                address, value
            ),
        }
    }
}
