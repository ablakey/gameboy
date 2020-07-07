use serde::Deserialize;

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Flags {
    Z: String,
    N: String,
    H: String,
    C: String,
}

#[derive(Deserialize, Debug)]
pub struct Operand {
    name: String,
    decrement: Option<bool>,
    increment: Option<bool>,
    immediate: bool,
}

#[derive(Deserialize, Debug)]
pub struct OpCode {
    mnemonic: String,
    bytes: u8,
    operands: Vec<Operand>,
    flags: Flags,
    cycles: Vec<u8>,
}

#[derive(Deserialize, Debug)]
pub struct OpCodes {
    unprefixed: HashMap<String, OpCode>,
    cbprefixed: HashMap<String, OpCode>,
}

impl OpCodes {
    /// Read opcode metadata from a JSON file.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let u = serde_json::from_reader(reader)?;
        Ok(u)
    }

    /// Get a string representation of an opcode. Great for debugging.const
    /// Examples:
    /// ```
    /// 0x31 LD   SP    d16    3 12    [- - - -]
    /// 0xAF XOR  A            1 4     [Z 0 0 0]
    /// 0x21 LD   HL    d16    3 12    [- - - -]
    /// 0x32 LD   (HL-) A      1 8     [- - - -]
    /// ```
    pub fn get_opcode_repr(&self, opcode_number: u8, is_cbprefix: bool) -> String {
        let opcode = self.get_opcode(opcode_number, is_cbprefix);

        /// Format an operand string given its parameters. For example: (HL-) is the HL register
        /// autodecrementing, with indirection.
        fn format_operand(operand: &Operand) -> String {
            let mut operand_str = String::from(&operand.name);

            if let Some(true) = operand.decrement {
                operand_str.push('-');
            }

            if let Some(false) = operand.increment {
                operand_str.push('+');
            }

            if !operand.immediate {
                operand_str = format! {"({})", operand_str};
            }

            format!("{:6}", operand_str)
        }

        let operand_strings: String = opcode
            .operands
            .iter()
            .map(format_operand)
            .collect::<Vec<String>>()
            .join("");

        let cycles = opcode
            .cycles
            .iter()
            .map(|c| format!("{}", c))
            .collect::<Vec<String>>()
            .join("/");

        format!(
            "{:#04X} {:4} {:12} {} {:5} [{} {} {} {}]",
            opcode_number,
            opcode.mnemonic,
            operand_strings,
            opcode.bytes,
            cycles,
            opcode.flags.Z,
            opcode.flags.N,
            opcode.flags.H,
            opcode.flags.C,
        )
    }

    /// Return the number of m-cycles (not t-states).
    /// The JSON stores t-states so we divide by four.
    /// See: https://gbdev.io/gb-opcodes/optables/ for details explaining m-cycles and t-states.
    /// action_taken is true if a conditional operation was undertaken that takes more CPU time to
    /// perform. There is always one cycle count, sometimes two.
    pub fn get_cycles(&self, opcode_number: u8, is_cbprefix: bool, action_taken: bool) -> u8 {
        let opcode = self.get_opcode(opcode_number, is_cbprefix);

        if action_taken {
            opcode.cycles[1] / 4
        } else {
            opcode.cycles[0] / 4
        }
    }

    /// Look up an opcode and return it.
    /// Panics if opcode was not found. This should never happen unless there's a bug in the
    /// emulator.
    fn get_opcode(&self, opcode_number: u8, is_cbprefix: bool) -> &OpCode {
        // Convert the hex opcode into a string representation as the map is keyed by strings.
        let opcode_string = format!("{:#04X}", opcode_number);

        let opcode_map = if is_cbprefix {
            &self.cbprefixed
        } else {
            &self.unprefixed
        };

        opcode_map
            .get(&opcode_string)
            .expect(format!("Could not find opcode: {}", opcode_string).as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_opcode() {
        let opcodes = OpCodes::from_path("data/opcodes.json").unwrap();

        let cycles = opcodes.get_cycles(0x00, false, false);
        assert_eq!(cycles, 1);
    }
}
