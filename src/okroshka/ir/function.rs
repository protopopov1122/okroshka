use crate::okroshka::ir::{IRIdentifier, IRBlock};

#[derive(Debug)]
pub struct IRFunctionDeclaration {
    id: IRIdentifier,
    name: Option<String>,
    params: IRIdentifier,
    vararg: bool,
    result: IRIdentifier
}

#[derive(Debug)]
pub struct IRFunction {
    name: String,
    declaration: IRIdentifier,
    locals: IRIdentifier,
    body: IRBlock
}


impl IRFunctionDeclaration {
    pub fn new(id: IRIdentifier, name: Option<String>, params: IRIdentifier, vararg: bool, result: IRIdentifier) -> IRFunctionDeclaration {
        IRFunctionDeclaration {
            id,
            name,
            params,
            vararg,
            result
        }
    }

    pub fn identifier(&self) -> IRIdentifier {
        self.id
    }

    pub fn name(&self) -> Option<&str> {
        match &self.name {
            Some(x) => Some(x.as_str()),
            None => None
        }
    }

    pub fn params_type(&self) -> IRIdentifier {
        self.params
    }

    pub fn params_vararg(&self) -> bool {
        self.vararg
    }

    pub fn return_type(&self) -> IRIdentifier {
        self.result
    }
}

impl IRFunction {
    pub fn new(name: String, declaration: IRIdentifier, locals: IRIdentifier, body: IRBlock) -> IRFunction {
        IRFunction {
            name,
            declaration,
            locals,
            body
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn declaration_id(&self) -> IRIdentifier {
        self.declaration
    }

    pub fn locals_type(&self) -> IRIdentifier {
        self.locals
    }

    pub fn body(&self) -> &IRBlock {
        &self.body
    }
}
