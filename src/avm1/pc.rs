use std::num::from_u8;
use std::mem::transmute;
use std::old_io::Reader;

use avm1::{ AVM1Error, AVM1ErrorKind, AVM1ActionKind, avm1_action_has_data };

#[derive(Debug, Copy)]
pub struct AVM1ProgramCounter<'a> {
    data: &'a [u8],
    pc: usize,
    operand: usize,
}

impl<'a> AVM1ProgramCounter<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        AVM1ProgramCounter {
            data: data,
            pc: 0,
            operand: 3
        }
    }

    pub fn increment(&mut self) -> bool {
        self.pc += self.len();
        self.reset()
    }

    pub fn offset(&mut self, offset: u16) -> bool {
        self.pc += offset as usize;
        self.reset()
    }

    fn reset(&mut self) -> bool {
        self.operand = self.pc + 3;
        self.is_valid()
    }

    pub fn is_valid(&self) -> bool {
        let len = self.data.len();
        self.pc < len && self.pc + self.len() <= len
    }

    pub fn opcode(&self) -> u8 {
        self.read_u8(self.pc)
    }

    pub fn action(&self) -> Option<AVM1ActionKind> {
        from_u8(self.opcode())
    }

    pub fn data_left(&self) -> usize {
        self.pc + self.len() - self.operand
    }

    fn len(&self) -> usize {
        match self.opcode() {
            op if avm1_action_has_data(op) => 3 + self.read_u16(self.pc + 1) as usize,
            _ => 1
        }
    }

    fn read_n(&self, offset: usize, len: usize) -> u64 {
        let mut value = 0u64;
        for i in 0..len {
            value |= (self.read_u8(offset + i) as u64) << (i * 8);
        }
        value
    }

    fn read_u8(&self, offset: usize) -> u8 {
        self.data[offset]
    }

    fn read_u16(&self, offset: usize) -> u16 {
        self.read_n(offset, 2) as u16
    }

    fn read_u32(&self, offset: usize) -> u32 {
        self.read_n(offset, 4) as u32
    }

    fn read_u64(&self, offset: usize) -> u64 {
        self.read_n(offset, 8) as u64
    }

    fn read_f32(&self, offset: usize) -> f32 {
        unsafe { transmute(self.read_u32(offset)) }
    }

    fn read_f64(&self, offset: usize) -> f64 {
        unsafe { transmute(self.read_u64(offset)) }
    }

    fn consume_operand(&mut self, sz: usize) -> bool {
        if self.operand + sz > self.data.len() { false } else { self.operand += sz; true }
    }

    pub fn operand_u8(&mut self) -> Result<u8, AVM1Error> {
        let operand = self.operand;
        if self.consume_operand(1) { Ok(self.read_u8(operand)) } else { operand_error() }
    }

    pub fn operand_u16(&mut self) -> Result<u16, AVM1Error> {
        let operand = self.operand;
        if self.consume_operand(2) { Ok(self.read_u16(operand)) } else { operand_error() }
    }

    pub fn operand_u32(&mut self) -> Result<u32, AVM1Error> {
        let operand = self.operand;
        if self.consume_operand(4) { Ok(self.read_u32(operand)) } else { operand_error() }
    }

    pub fn operand_f32(&mut self) -> Result<f32, AVM1Error> {
        let operand = self.operand;
        if self.consume_operand(4) { Ok(self.read_f32(operand)) } else { operand_error() }
    }

    pub fn operand_f64(&mut self) -> Result<f64, AVM1Error> {
        let operand = self.operand;
        if self.consume_operand(8) { Ok(self.read_f64(operand)) } else { operand_error() }
    }

    pub fn operand_string(&mut self) -> Result<String, AVM1Error> {
        let mut operand = self.operand;
        let mut value = Vec::new();
        loop {
            if self.consume_operand(1) {
                let v = self.read_u8(operand);
                if v == 0 {
                    break
                }
                value.push(v);
                operand += 1;
            } else {
                return operand_error()
            }
        }
        if let Ok(value) = String::from_utf8(value) {
            Ok(value)
        } else {
            operand_error()
        }
    }
}

fn operand_error<T>() -> Result<T, AVM1Error> {
    Err(AVM1Error::new(AVM1ErrorKind::InvalidOperand))
}
