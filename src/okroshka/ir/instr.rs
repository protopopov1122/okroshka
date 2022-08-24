use crate::okroshka::ir::{
    IRTypeRef
};

#[derive(Debug, Clone, Copy)]
pub struct IRInstructionMemFlags {
    pub volatile: bool
}

#[derive(Debug)]
pub enum IRInstructionArgument<'a> {
    None,
    Integer(i64),
    UInteger(u64),
    UIntegerPair(u32, u32),
    Boolean(bool),
    Float64(f64),
    Float32(f32),
    String(u64),
    TypeRef(IRTypeRef),
    CodeRef(usize),
    Identifier(&'a str),
    FunctionRef(u64, Option<&'a String>),
    MemFlags(IRInstructionMemFlags)
}

include!(concat!(env!("OUT_DIR"), "/opcodes.rs"));