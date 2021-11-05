pub struct Opcode {
    pub digits: [u16; 4],       // Keep as u16 values instead of u8 to make some
}                               // math operations easier/less messy

impl Opcode {
    pub fn new(opcode: &[u8]) -> Opcode {
        return Opcode {
            digits: [
                u16::from((opcode[0]) & 0xF0) >> 4,
                u16::from(opcode[0]) & 0x0F,
                u16::from((opcode[1]) & 0xF0) >> 4,
                u16::from(opcode[1]) & 0x0F,
            ]
        };
    }
}