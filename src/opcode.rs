pub struct Opcode {
    pub digits: [u16; 4],
}

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

    pub fn display(self: &Self){
        println!("{:01X}{:01X}{:01X}{:01X}", 
                    self.digits[0], self.digits[1], self.digits[2], self.digits[3]);
    }
}