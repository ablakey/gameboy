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

    /// Get a string representation of an opcode. Great for debugging
    pub fn get_opcode_repr(&self, opcode: u8, is_cbprefix: bool) -> String {
        let opcode_map = if is_cbprefix {
            &self.cbprefixed
        } else {
            &self.unprefixed
        };

        // Convert the hex opcode into a string representation as the map is keyed by strings.
        let opcode_string = format!("{:#04X}", opcode);

        let opcode = opcode_map
            .get(&opcode_string)
            .expect(format!("Could not find opcode: {}", opcode_string).as_str());

        let operand_strings = opcode
            .operands
            .iter()
            .map(|o| o.name.as_str())
            .collect::<Vec<&str>>()
            .join(", ");

        let cycles = opcode
            .cycles
            .iter()
            .map(|c| format!("{}", c))
            .collect::<Vec<String>>()
            .join("/");

        format!(
            "{} {} {} [{} {}] [{} {} {} {}]",
            opcode_string,
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
}
