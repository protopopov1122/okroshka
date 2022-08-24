use std::vec::Vec;

use crate::okroshka::ir::core::IRIdentifier;

#[derive(Debug)]
pub enum IRTypeBuiltin {
    VarargList
}

#[derive(Debug)]
pub enum IRTypeEntry {
    Struct { alignment: Option<u64>, num_of_fields: usize },
    Array { alignment: Option<u64>, length: u64 },
    Union { alignment: Option<u64>, num_of_fields: usize },
    Int8 { alignment: Option<u64> },
    Int16 { alignment: Option<u64> },
    Int32 { alignment: Option<u64> },
    Int64 { alignment: Option<u64> },
    Float32 { alignment: Option<u64> },
    Float64 { alignment: Option<u64> },
    LongDouble { alignment: Option<u64> },
    Bool { alignment: Option<u64> },
    Char { alignment: Option<u64> },
    Short { alignment: Option<u64> },
    Int { alignment: Option<u64> },
    Long { alignment: Option<u64> },
    Word { alignment: Option<u64> },
    Bits { alignment: Option<u64>, width: u64 },
    Builtin { alignment: Option<u64>, builtin: IRTypeBuiltin }
}

#[derive(Debug)]
pub struct IRType {
    id: IRIdentifier,
    content: Vec<IRTypeEntry>
}

#[derive(Debug, Clone, Copy)]
pub struct IRTypeRef {
    pub type_id: IRIdentifier,
    pub type_index: usize
}

impl IRTypeRef {
    pub fn new(type_id: IRIdentifier, type_index: usize) -> IRTypeRef {
        IRTypeRef {
            type_id,
            type_index
        }
    }
}

impl IRType {
    pub fn new(id: IRIdentifier, content: Vec<IRTypeEntry>) -> IRType {
        IRType {
            id,
            content
        }
    }

    pub fn identifier(&self) -> IRIdentifier {
        self.id
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn at(&self, index: u64) -> Option<&IRTypeEntry> {
        self.content.get(index as usize)
    }

    pub fn type_entries(&self) -> impl Iterator<Item = &IRTypeEntry> {
        self.content.iter()
    }
}