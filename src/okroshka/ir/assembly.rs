use std::collections::HashSet;
use std::collections::HashMap;
use std::iter::Iterator;

use crate::okroshka::ir::{IRIdentifier, IRTypeRef};

use super::IRError;

#[derive(Debug, Clone)]
pub enum IRInlineAssemblyParameterClass {
    ImmediateConstant(IRTypeRef, i64),
    ImmediateIdentifierBased(IRTypeRef, String, i64),
    ImmediateLiteralBased(IRTypeRef, IRIdentifier, i64),
    Read(IRTypeRef, u64),
    Load(IRTypeRef, u64),
    Store(IRTypeRef, u64),
    LoadStore(IRTypeRef, u64),
    ReadStore(IRTypeRef, u64, IRTypeRef, u64)
}

#[derive(Debug, Copy, Clone)]
pub enum IRInlineAssemblyParameterConstraint {
    None,
    Register,
    Memory,
    RegisterMemory
}

#[derive(Debug)]
pub struct IRInlineAssemblyParameter {
    id: IRIdentifier,
    aliases: Vec<String>,
    klass: IRInlineAssemblyParameterClass,
    constraint: IRInlineAssemblyParameterConstraint
}

#[derive(Debug)]
pub struct IRInlineAssemblyJumpTarget {
    id: IRIdentifier,
    aliases: Vec<String>,
    target_function: String,
    target_offset: usize
}

#[derive(Debug, Copy, Clone)]
pub enum IRInlineAssemblyIndexedAlias {
    Parameter(IRIdentifier),
    JumpTarget(IRIdentifier)
}

#[derive(Debug)]
pub struct IRInlineAssembly {
    id: IRIdentifier,
    global: bool,
    template: String,
    parameters: HashMap<IRIdentifier, IRInlineAssemblyParameter>,
    clobbers: HashSet<String>,
    jump_targets: HashMap<IRIdentifier, IRInlineAssemblyJumpTarget>,
    alias_index: HashMap<String, IRInlineAssemblyIndexedAlias>
}

impl IRInlineAssemblyParameter {
    pub fn new(id: IRIdentifier, aliases: Vec<String>, klass: IRInlineAssemblyParameterClass, constraint: IRInlineAssemblyParameterConstraint) -> IRInlineAssemblyParameter {
        IRInlineAssemblyParameter {
            id,
            aliases,
            klass,
            constraint
        }
    }

    pub fn identifier(&self) -> IRIdentifier {
        self.id
    }

    pub fn aliases(&self) -> impl Iterator<Item=&str> {
        self.aliases.iter()
            .map(| s | s.as_str())
    }

    pub fn klass(&self) -> &IRInlineAssemblyParameterClass {
        &self.klass
    }

    pub fn constraint(&self) -> IRInlineAssemblyParameterConstraint {
        self.constraint
    }
}

impl IRInlineAssemblyJumpTarget {
    pub fn new(id: IRIdentifier, aliases: Vec<String>, target_function: String, target_offset: usize) -> IRInlineAssemblyJumpTarget {
        IRInlineAssemblyJumpTarget {
            id,
            aliases,
            target_function,
            target_offset
        }
    }

    pub fn identifier(&self) -> IRIdentifier {
        self.id
    }

    pub fn aliases(&self) -> impl Iterator<Item = &str> {
        self.aliases.iter()
            .map(| s | s.as_str())
    }

    pub fn target_function(&self) -> &str {
        self.target_function.as_str()
    }

    pub fn target_function_offset(&self) -> usize {
        self.target_offset
    }
}

impl IRInlineAssembly {
    pub fn new(id: IRIdentifier, global: bool, template: String, parameters: HashMap<IRIdentifier, IRInlineAssemblyParameter>, clobbers: HashSet<String>, jump_targets: HashMap<IRIdentifier, IRInlineAssemblyJumpTarget>) -> Result<IRInlineAssembly, IRError> {
        let mut inline_asm = IRInlineAssembly {
            id,
            global,
            template,
            parameters,
            clobbers,
            jump_targets,
            alias_index: HashMap::new()
        };

        for (&param_id, param) in inline_asm.parameters.iter() {
            if param_id != param.identifier() {
                Err(IRError("IR inline assembly parameter identifier does not match the index".to_owned()))?;
            }
            for alias in param.aliases() {
                if inline_asm.alias_index.insert(alias.to_owned(), IRInlineAssemblyIndexedAlias::Parameter(param_id)).is_some() {
                    Err(IRError("Detected duplicating IR inline assembly aliases".to_owned()))?;
                }
            }
        }

        for (&target_id, target) in inline_asm.jump_targets.iter() {
            if target_id != target.identifier() {
                Err(IRError("IR inline assembly parameter identifier does not match the index".to_owned()))?;
            }
            for alias in target.aliases() {
                if inline_asm.alias_index.insert(alias.to_owned(), IRInlineAssemblyIndexedAlias::JumpTarget(target_id)).is_some() {
                    Err(IRError("Detected duplicating IR inline assembly aliases".to_owned()))?;
                }
            }
        }

        Ok(inline_asm)
    }

    pub fn identifier(&self) -> IRIdentifier {
        self.id
    }

    pub fn is_global(&self) -> bool {
        self.global
    }

    pub fn template(&self) -> &str {
        self.template.as_str()
    }

    pub fn get_parameter(&self, id: IRIdentifier) -> Option<&IRInlineAssemblyParameter> {
        self.parameters.get(&id)
    }

    pub fn parameters(&self) -> impl Iterator<Item=&IRInlineAssemblyParameter> {
        self.parameters
            .iter()
            .map(| (_, param) | param)
    }

    pub fn has_clobber(&self, clobber: &str) -> bool {
        self.clobbers.contains(clobber)
    }

    pub fn clobbers(&self) -> impl Iterator<Item=&str> {
        self.clobbers.iter()
            .map(| x | x.as_str())
    }

    pub fn get_jump_target(&self, id: IRIdentifier) -> Option<&IRInlineAssemblyJumpTarget> {
        self.jump_targets.get(&id)
    }

    pub fn jump_targets(&self) -> impl Iterator<Item=&IRInlineAssemblyJumpTarget> {
        self.jump_targets
            .iter()
            .map(| (_, target) | target)
    }

    pub fn get_by_alias(&self, alias: &str) -> Option<IRInlineAssemblyIndexedAlias> {
        self.alias_index.get(alias).map(| x | *x)
    }
}