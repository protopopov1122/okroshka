use std::vec::Vec;

use crate::okroshka::ir::instr::IRInstruction;

#[derive(Debug)]
pub struct IRBlock {
    code: Vec<IRInstruction>
}

impl IRBlock {
    pub fn new(code: Vec<IRInstruction>) -> IRBlock {
        IRBlock {
            code
        }
    }

    pub fn code(&self) -> impl Iterator<Item = &IRInstruction> {
        self.code.iter()
    }

    pub fn at(&self, index: usize) -> Option<&IRInstruction> {
        self.code.get(index)
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }
}