use std::vec::Vec;

use crate::okroshka::ir::IRIdentifier;

#[derive(Debug, Copy, Clone)]
pub enum IRDataStorage {
    Global,
    ThreadLocal
}

#[derive(Debug)]
pub enum IRDataElement {
    Undefined(u64),
    Integer(i64),
    Float32(f32),
    Float64(f64),
    LongDouble(f64),
    String(Vec<u8>),
    Pointer{ base: String, offset: i64 },
    StringPointer{base: IRIdentifier, offset: i64},
    Raw(Vec<u8>),
    Aggregate
}

#[derive(Debug)]
pub struct IRData {
    name: String,
    storage: IRDataStorage,
    datatype: IRIdentifier,
    data: Vec<IRDataElement>
}

impl IRData {
    pub fn new(name: String, storage: IRDataStorage, datatype: IRIdentifier, data: Vec<IRDataElement>) -> IRData {
        IRData {
            name,
            storage,
            datatype,
            data
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn data_type(&self) -> IRIdentifier {
        self.datatype
    }

    pub fn data_storage(&self) -> IRDataStorage {
        self.storage
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn at(&self, index: u64) -> Option<&IRDataElement> {
        self.data.get(index as usize)
    }

    pub fn elements(&self) -> impl Iterator<Item = &IRDataElement> {
        self.data.iter()
    }
}
