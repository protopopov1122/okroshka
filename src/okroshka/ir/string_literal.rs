use std::vec::Vec;

use crate::okroshka::ir::IRIdentifier;

#[derive(Debug)]
pub enum IRStringLiteralContent {
    Multibyte(Vec<u8>),
    Unicode16(Vec<u16>),
    Unicode32(Vec<u32>)
}

#[derive(Debug)]
pub struct IRStringLiteral {
    id: IRIdentifier,
    public: bool,
    content: IRStringLiteralContent
}

impl IRStringLiteral {
    pub fn new(id: IRIdentifier, public: bool, content: IRStringLiteralContent) -> IRStringLiteral {
        IRStringLiteral {
            id,
            public,
            content
        }
    }

    pub fn identifier(&self) -> IRIdentifier {
        self.id
    }

    pub fn is_public(&self) -> bool {
        self.public
    }

    pub fn content(&self) -> &IRStringLiteralContent {
        &self.content
    }
}
