use std::collections::HashMap;

use crate::okroshka::ir::{
    IRIdentifier,
    IRError,
    IRType,
    IRTypeRef,
    IRFunctionDeclaration,
    IRFunction,
    IRSymbol,
    IRStringLiteral,
    IRData,
    IRInlineAssembly,
    IRInlineAssemblyParameterClass,
    IRBlock,
    IRInstructionArgument
};

#[derive(Debug)]
pub struct IRModule {
    globals: HashMap<String, IRSymbol>,
    externals: HashMap<String, IRSymbol>,
    types: HashMap<IRIdentifier, IRType>,
    string_literals: HashMap<IRIdentifier, IRStringLiteral>,
    function_declarations: HashMap<IRIdentifier, IRFunctionDeclaration>,
    functions: HashMap<String, IRFunction>,
    data: HashMap<String, IRData>,
    inline_asm: HashMap<IRIdentifier, IRInlineAssembly>
}

impl IRModule {
    pub fn new(globals: HashMap<String, IRSymbol>,
               externals: HashMap<String, IRSymbol>,
               types: HashMap<IRIdentifier, IRType>,
               string_literals: HashMap<IRIdentifier, IRStringLiteral>,
               function_declarations: HashMap<IRIdentifier, IRFunctionDeclaration>,
               functions: HashMap<String, IRFunction>,
               data: HashMap<String, IRData>,
               inline_asm: HashMap<IRIdentifier, IRInlineAssembly>) -> Result<IRModule, IRError> {
        let module = IRModule {
            globals,
            externals,
            types,
            string_literals,
            function_declarations,
            functions,
            data,
            inline_asm
        };
        module.check()?;
        Ok(module)
    }

    pub fn globals(&self) -> impl Iterator<Item = &IRSymbol> {
        self.globals.iter()
            .map(| (_, value) | value)
    }

    pub fn is_global(&self, sym: &str) -> bool {
        self.globals.contains_key(sym)
    }

    pub fn externals(&self) -> impl Iterator<Item = &IRSymbol> {
        self.externals.iter()
            .map(| (_, value) | value)
    }

    pub fn is_external(&self, sym: &str) -> bool {
        self.externals.contains_key(sym)
    }

    pub fn types(&self) -> impl Iterator<Item = &IRType> {
        self.types.iter()
            .map(| (_, tp) | tp)
    }

    pub fn get_type(&self, id: IRIdentifier) -> Option<&IRType> {
        self.types.get(&id)
    }

    pub fn string_literals(&self) -> impl Iterator<Item = &IRStringLiteral>{
        self.string_literals.iter()
            .map(| (_, lit) | lit)
    }

    pub fn get_string_literal(&self, id: IRIdentifier) -> Option<&IRStringLiteral>{
        self.string_literals.get(&id)
    }

    pub fn get_function_declaration(&self, id: IRIdentifier) -> Option<&IRFunctionDeclaration> {
        self.function_declarations.get(&id)
    }

    pub fn function_declarations(&self) -> impl Iterator<Item = &IRFunctionDeclaration> {
        self.function_declarations.iter()
            .map(| (_, decl) | decl)
    }

    pub fn get_function(&self, name: &str) -> Option<&IRFunction> {
        self.functions.get(name)
    }

    pub fn functions(&self) -> impl Iterator<Item = &IRFunction> {
        self.functions.iter()
            .map(| (_, func) | func)
    }

    pub fn get_data(&self, id: &str) -> Option<&IRData> {
        self.data.get(id)
    }

    pub fn data(&self) -> impl Iterator<Item = &IRData> {
        self.data.iter()
            .map(| (_, data) | data)
    }

    pub fn get_inline_assembly(&self, id: IRIdentifier) -> Option<&IRInlineAssembly> {
        self.inline_asm.get(&id)
    }

    pub fn inline_assembly(&self) -> impl Iterator<Item = &IRInlineAssembly> {
        self.inline_asm.iter()
            .map(| (_, inline_asm) | inline_asm)
    }

    fn check(&self) -> Result<(), IRError> {
        for (&func_decl_id, func_decl) in self.function_declarations.iter() {
            if func_decl_id != func_decl.identifier() {
                Err(IRError("Expected IR function declaration identifier to match respective index key".to_owned()))?;
            }

            self.check_type_id(func_decl.params_type())?;
            self.check_type_id(func_decl.return_type())?;
        }

        for (func_id, func) in self.functions.iter() {
            if func_id != func.name() {
                Err(IRError("Expected IR function declaration identifier to match respective index key".to_owned()))?;
            }

            self.check_function_declaration_id(func.declaration_id())?;
            self.check_type_id(func.locals_type())?;
            self.check_block(func.body())?;
        }

        for (data_name, data) in self.data.iter() {
            if data_name != data.name() {
                Err(IRError("Expected IR data name to match respective index key".to_owned()))?;
            }

            self.check_type_id(data.data_type())?;
        }

        for (&inline_asm_id, inline_asm) in self.inline_asm.iter() {
            if inline_asm_id != inline_asm.identifier() {
                Err(IRError("Expected IR inline assembly identifier to match respective index key".to_owned()))?;
            }

            for param in inline_asm.parameters() {
                match param.klass() {
                    IRInlineAssemblyParameterClass::ImmediateConstant(typeref, _)
                        => self.check_type_ref(*typeref)?,

                    IRInlineAssemblyParameterClass::ImmediateIdentifierBased(typeref, _, _)
                    => self.check_type_ref(*typeref)?,

                    IRInlineAssemblyParameterClass::ImmediateLiteralBased(typeref, literal_id, _)
                        => {
                        self.check_type_ref(*typeref)?;
                        self.check_string_literal(*literal_id)?;
                    }

                    IRInlineAssemblyParameterClass::Read(typeref, _)
                        => self.check_type_ref(*typeref)?,

                    IRInlineAssemblyParameterClass::Load(typeref, _)
                        => self.check_type_ref(*typeref)?,
                    
                    IRInlineAssemblyParameterClass::Store(typeref, _)
                        => self.check_type_ref(*typeref)?,
                        
                    IRInlineAssemblyParameterClass::LoadStore(typeref, _)
                        => self.check_type_ref(*typeref)?,
                    
                    IRInlineAssemblyParameterClass::ReadStore(typeref1, _, typeref2, _)
                        => {
                        self.check_type_ref(*typeref1)?;
                        self.check_type_ref(*typeref2)?;
                    }
                };
            }

            for jump_target in inline_asm.jump_targets() {
                let func = self.get_function(jump_target.target_function())
                    .ok_or(IRError("Unable to find specified jump target function".to_owned()))?;
                if jump_target.target_function_offset() > func.body().len() {
                    Err(IRError("Expected IR inline assembly jump target offset exceeds respective function body length".to_owned()))?;
                }
            }
        }

        Ok(())
    }

    fn check_block(&self, block: &IRBlock) -> Result<(), IRError> {
        for instr in block.code() {
            match instr.argument() {
                IRInstructionArgument::CodeRef(coderef)
                    => if coderef > block.len() {
                    Err(IRError("IR instruction argument code reference exceeds respective block boundaries".to_owned()))?
                },
                IRInstructionArgument::String(str_id)
                    => self.check_string_literal(str_id)?,
                IRInstructionArgument::TypeRef(typeref)
                    => self.check_type_ref(typeref)?,
                IRInstructionArgument::FunctionRef(decl_id, _)
                    => self.check_function_declaration_id(decl_id)?,
                _ => ()
            }
        }

        Ok(())
    }

    fn check_function_declaration_id(&self, id: IRIdentifier) -> Result<(), IRError> {
        match self.function_declarations.get(&id) {
            Some(_) => Ok(()),
            None => Err(IRError("Provided IR function declaration identifier does not exist in the module".to_owned()))
        }
    }

    fn check_string_literal(&self, id: IRIdentifier) -> Result<(), IRError> {
        match self.string_literals.get(&id) {
            Some(_) => Ok(()),
            None => Err(IRError("Provided IR string literal identifier does not exist in the module".to_owned()))
        }
    }

    fn check_type_id(&self, id: IRIdentifier) -> Result<(), IRError> {
        match self.types.get(&id) {
            Some(_) => Ok(()),
            None => Err(IRError("Provided IR type identifier does not exist in the module".to_owned()))
        }
    }

    fn check_type_ref(&self, typeref: IRTypeRef) -> Result<(), IRError> {
        match self.types.get(&typeref.type_id) {
            Some(tp) => if typeref.type_index < tp.len() {
                Ok(())
            } else {
                Err(IRError("Provided IR type index exceeds respective IR type length".to_owned()))
            },
            None => Err(IRError("Provided IR type identifier does not exist in the module".to_owned()))
        }
    }
}
