use std::collections::{HashMap, HashSet};
use std::vec::Vec;
use serde::{Deserialize, Deserializer};
use serde::de::{Error, IntoDeserializer};
use serde_json::Value;

use crate::okroshka::ir::{
    IRSymbol,
    IRIdentifier,
    IRBlock,
    IRType,
    IRTypeEntry,
    IRTypeBuiltin,
    IRFunctionDeclaration,
    IRFunction,
    IRData,
    IRDataElement,
    IRDataStorage,
    IRStringLiteral,
    IRModule,
    IRStringLiteralContent,
    IRInlineAssembly,
    IRInlineAssemblyParameterClass,
    IRTypeRef,
    IRInlineAssemblyParameterConstraint,
    IRInlineAssemblyParameter,
    IRInlineAssemblyJumpTarget,
    IRInstructionMemFlags,
    IRInstruction
};

include!(concat!(env!("OUT_DIR"), "/instr-loader.rs"));

impl<'de> Deserialize<'de> for IRSymbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let identifier = value.get("identifier")
            .map(| val | val.as_str())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR symbol identifier"))?;
        match value.get("type").map(| val | val.as_str()).flatten() {
            Some("global") => Ok(IRSymbol::Global(identifier.to_owned())),
            Some("thread_local") => Ok(IRSymbol::ThreadLocal(identifier.to_owned())),
            _ => Err(D::Error::custom("unable to deserialize IR symbol type"))
        }
    }
}

fn deserialize_instr_u64<'de, D>(value: &Value) -> Result<u64, D::Error>
where D: Deserializer<'de> {
    match value.get("arg") {
        Some(serde_json::Value::Number(x)) if x.is_u64() =>
            Ok(x.as_u64().unwrap()),
        _ => Err(D::Error::custom("unable to deserialize IR instruction argument"))
    }
}

fn deserialize_instr_usize<'de, D>(value: &Value) -> Result<usize, D::Error>
where D: Deserializer<'de> {
    match value.get("arg") {
        Some(serde_json::Value::Number(x)) if x.is_u64() =>
            Ok(x.as_u64().unwrap() as usize),
        _ => Err(D::Error::custom("unable to deserialize IR instruction argument"))
    }
}

fn deserialize_instr_i64<'de, D>(value: &Value) -> Result<i64, D::Error>
where D: Deserializer<'de> {
    match value.get("arg") {
        Some(serde_json::Value::Number(x)) if x.is_i64() =>
            Ok(x.as_i64().unwrap()),
        _ => Err(D::Error::custom("unable to deserialize IR instruction argument"))
    }
}

fn deserialize_instr_f64<'de, D>(value: &Value) -> Result<f64, D::Error>
where D: Deserializer<'de> {
    match value.get("arg") {
        Some(serde_json::Value::Number(x)) if x.is_f64() =>
            Ok(x.as_f64().unwrap()),
        _ => Err(D::Error::custom("unable to deserialize IR instruction argument"))
    }
}

fn deserialize_instr_f32<'de, D>(value: &Value) -> Result<f32, D::Error>
where D: Deserializer<'de> {
    match value.get("arg") {
        Some(serde_json::Value::Number(x)) if x.is_f64() =>
            Ok(x.as_f64().unwrap() as f32),
        _ => Err(D::Error::custom("unable to deserialize IR instruction argument"))
    }
}

fn deserialize_instr_u32<'de, D>(value: &Value) -> Result<(u32, u32), D::Error>
where D: Deserializer<'de> {
    match value.get("arg") {
        Some(serde_json::Value::Array(x)) if x.len() == 2 && x[0].is_u64() && x[1].is_u64() => {
            let type_id = x[0].as_u64().ok_or(D::Error::custom("unable to deserialize IR instruction argument"))?;
            let type_index = x[1].as_u64().ok_or(D::Error::custom("unable to deserialize IR instruction argument"))?;
            Ok((type_id as u32, type_index as u32))
        },
        _ => Err(D::Error::custom("unable to deserialize IR instruction argument"))
    }
}

fn deserialize_instr_bool<'de, D>(value: &Value) -> Result<bool, D::Error>
where D: Deserializer<'de> {
    match value.get("arg") {
        Some(serde_json::Value::Bool(x)) => Ok(*x),
        _ => Err(D::Error::custom("unable to deserialize IR instruction argument"))
    }
}

fn deserialize_instr_typeref<'de, D>(value: &Value) -> Result<IRTypeRef, D::Error>
where D: Deserializer<'de> {
    match value.get("arg") {
        Some(x) => {
            let type_id = x.get("type")
                .map(| val | val.as_u64())
                .flatten()
                .ok_or(D::Error::custom("unable to deserialize IR instruction argument"))?;
            let type_index = x.get("index")
                .map(| val | val.as_u64())
                .flatten()
                .ok_or(D::Error::custom("unable to deserialize IR instruction argument"))? as usize;
            Ok(IRTypeRef::new(type_id, type_index))
        },
        _ => Err(D::Error::custom("unable to deserialize IR instruction argument"))
    }
}

fn deserialize_instr_identifier<'de, D>(value: &Value) -> Result<String, D::Error>
where D: Deserializer<'de> {
    match value.get("arg") {
        Some(x) if x.get("data").is_some() => {
            let identifier = x.get("data")
                .map(| val | val.as_str())
                .flatten()
                .ok_or(D::Error::custom("unable to deserialize IR instruction argument"))?.to_owned();
            Ok(identifier)
        },
        _ => Err(D::Error::custom("unable to deserialize IR instruction argument"))
    }
}

fn deserialize_instr_funcref<'de, D>(value: &Value) -> Result<(u64, Option<String>), D::Error>
where D: Deserializer<'de> {
    match value.get("arg") {
        Some(x) if x.get("identifier").is_some() => {
            let identifier = x.get("identifier")
                .map(| val | val.as_u64())
                .flatten()
                .ok_or(D::Error::custom("unable to deserialize IR instruction argument"))?;
            let name: Option<String> = match x.get("name") {
                Some(serde_json::Value::String(x)) => Some(x.to_owned()),
                Some(serde_json::Value::Null) => None,
                None => None,
                _ => Err(D::Error::custom("unable to deserialize IR instruction argument"))?
            };
            Ok((identifier, name))
        },
        _ => Err(D::Error::custom("unable to deserialize IR instruction argument"))
    }
}

fn deserialize_instr_memflags<'de, D>(value: &Value) -> Result<IRInstructionMemFlags, D::Error>
where D: Deserializer<'de> {
    match value.get("memory_flags") {
        Some(x) if x.is_object() => {
            let volatile_flag = x.get("volatile")
                .map(| val | val.as_bool())
                .flatten()
                .ok_or(D::Error::custom("unable to deserialize IR instruction memory flags"))?;
            Ok(IRInstructionMemFlags {
                volatile: volatile_flag
            })
        },
        _ => Err(D::Error::custom("unable to deserialize IR instruction argument"))
    }
}

impl<'de> Deserialize<'de> for IRInstruction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let opcode = value.get("opcode")
            .map(| val | val.as_str())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR instruction opcode"))?;
        deserialize_instruction::<D>(opcode, &value)
    }
}

impl<'de> Deserialize<'de> for IRBlock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let instrs: Result<Vec<IRInstruction>, D::Error> = match value.as_array() {
            Some(arr) => arr.iter()
                .map(| instr_value | IRInstruction::deserialize(instr_value.clone().into_deserializer()).map_err(D::Error::custom))
                .collect(),
            None => Err(D::Error::custom("unable to deserialize IR block"))
        };
        Ok(IRBlock::new(instrs?))
    }
}

impl<'de> IRType {
    fn deserialize_typeentry_array<D, F>(deserializer: D, output_fn: &mut F) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
        F: FnMut(IRTypeEntry) -> ()
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        value.as_array()
            .ok_or(D::Error::custom("unable to deserialize IR type"))?
            .iter()
            .try_for_each(| typeentry_value | IRType::deserialize_typeentry(typeentry_value, output_fn).map_err(D::Error::custom))?;
        Ok(())
    }

    fn deserialize_typeentry<D, F>(deserializer: D, output_fn: &mut F) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
        F: FnMut(IRTypeEntry) -> ()
    {
        let typeentry_value = serde_json::Value::deserialize(deserializer)?;
        let alignment = typeentry_value.get("alignment").map(| val | val.as_u64()).flatten();
        match typeentry_value.get("type").map(| val | val.as_str()).flatten(){
            Some("int8") => output_fn(IRTypeEntry::Int8{alignment}),
            Some("int16") => output_fn(IRTypeEntry::Int16{alignment}),
            Some("int32") => output_fn(IRTypeEntry::Int32{alignment}),
            Some("int64") => output_fn(IRTypeEntry::Int64{alignment}),
            Some("bool") => output_fn(IRTypeEntry::Bool{alignment}),
            Some("char") => output_fn(IRTypeEntry::Char{alignment}),
            Some("short") => output_fn(IRTypeEntry::Short{alignment}),
            Some("int") => output_fn(IRTypeEntry::Int{alignment}),
            Some("long") => output_fn(IRTypeEntry::Long{alignment}),
            Some("word") => output_fn(IRTypeEntry::Word{alignment}),
            Some("float") => output_fn(IRTypeEntry::Float32{alignment}),
            Some("double") => output_fn(IRTypeEntry::Float64{alignment}),
            Some("long_double") => output_fn(IRTypeEntry::LongDouble{alignment}),
            Some("bits") => output_fn(IRTypeEntry::Bits{
                alignment,
                width: typeentry_value.get("width")
                    .map(| val | val.as_u64())
                    .flatten()
                    .ok_or(D::Error::custom("unable to deserialize IR type entry"))?
            }),
            Some("builtin") => output_fn(match typeentry_value.get("class").map(| val | val.as_str()).flatten() {
                Some("vararg") => IRTypeEntry::Builtin{
                    alignment,
                    builtin: IRTypeBuiltin::VarargList
                },
                _ => Err(D::Error::custom("unable to deserialize IR type entry"))?
            }),
            Some("struct") => {
                let fields = typeentry_value.get("fields")
                    .ok_or(D::Error::custom("unable to deserialize IR type entry"))?;
                output_fn(IRTypeEntry::Struct{
                    alignment,
                    num_of_fields: fields
                        .as_array()
                        .ok_or(D::Error::custom("unable to deserialize IR type entry"))?
                        .len()
                });
                IRType::deserialize_typeentry_array( fields.clone().into_deserializer(), output_fn).map_err(D::Error::custom)?;
            },
            Some("union") => {
                let fields = typeentry_value.get("fields")
                    .ok_or(D::Error::custom("unable to deserialize IR type entry"))?;
                output_fn(IRTypeEntry::Union{
                    alignment,
                    num_of_fields: fields
                        .as_array()
                        .ok_or(D::Error::custom("unable to deserialize IR type entry"))?
                        .len()
                });
                IRType::deserialize_typeentry_array(fields.clone().into_deserializer(), output_fn).map_err(D::Error::custom)?;
            },
            Some("array") => {
                output_fn(IRTypeEntry::Array{
                    alignment,
                    length: typeentry_value.get("length")
                        .map(| val | val.as_u64())
                        .flatten()
                        .ok_or(D::Error::custom("unable to deserialize IR type entry"))?
                });
                IRType::deserialize_typeentry(
                    typeentry_value.get("element_type")
                        .ok_or(D::Error::custom("unable to deserialize IR type entry".to_owned()))?
                        .clone()
                        .into_deserializer(),
                    output_fn).map_err(D::Error::custom)?;
            },
            _ => Err(D::Error::custom("unable to deserialize IR type entry"))?
        };
        Ok(())
    }
}

impl<'de> Deserialize<'de> for IRType {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let identifier = value.get("identifier")
            .map(| val | val.as_u64())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR type"))?;
        let mut type_content = Vec::new();
        IRType::deserialize_typeentry_array(
            value.get("type")
                .ok_or(D::Error::custom("unable to deserialize IR type".to_owned()))?
                .clone()
                .into_deserializer(), 
            &mut | x | type_content.push(x)).map_err(D::Error::custom)?;
        Ok(IRType::new(identifier, type_content))
    }
}

impl<'de> Deserialize<'de> for IRFunctionDeclaration {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let identifier = value.get("identifier")
            .map(| val | val.as_u64())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR function declaration identifier"))?;
        let name = value.get("name")
            .map(| val | val.as_str())
            .flatten()
            .map(| x | x.to_owned());
        let params = value.get("parameters")
            .map(| val | val.as_u64())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR function declaration parameters"))?;
        let vararg = value.get("vararg")
            .map(| val | val.as_bool())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR function declaration vararg"))?;
        let returns = value.get("returns")
            .map(| val | val.as_u64())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR function declaration identifier"))?;
        Ok(IRFunctionDeclaration::new(identifier, name, params, vararg, returns))
    }
}

impl<'de> Deserialize<'de> for IRFunction {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let identifier = value.get("identifier")
            .map(| val | val.as_u64())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR function identifier"))?;
        let name = value.get("name")
            .map(| val | val.as_str())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR function name"))?.to_owned();
        let locals = value.get("locals")
            .map(| val | val.as_u64())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR function locals"))?;
        let body = IRBlock::deserialize(
            value.get("body")
                .ok_or(D::Error::custom("unable to deserialize IR function body"))?
                .clone()
                .into_deserializer())
            .map_err(D::Error::custom)?;
        Ok(IRFunction::new(name, identifier, locals, body))
    }
}

impl<'de> Deserialize<'de> for IRDataElement {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let data_entry = match value.get("class").map(| val | val.as_str()).flatten() {
            Some("undefined") => IRDataElement::Undefined(
                value.get("count")
                    .map(| val | val.as_u64())
                    .flatten()
                    .unwrap_or(1)),
            Some("aggregate") => IRDataElement::Aggregate,
            Some("integer") => IRDataElement::Integer(
                value.get("value")
                    .map(| val | val.as_i64())
                    .flatten()
                    .ok_or(D::Error::custom("unable to deserialize IR integral data value"))?),
            Some("float32") => IRDataElement::Float32(
                value.get("value")
                    .map(| val | val.as_f64())
                    .flatten()
                    .map(| x | x as f32)
                    .ok_or(D::Error::custom("unable to deserialize IR floating-point data value"))?),
            Some("float64") => IRDataElement::Float64(
                value.get("value")
                    .map(| val | val.as_f64())
                    .flatten()
                    .ok_or(D::Error::custom("unable to deserialize IR floating-point data value"))?),
            Some("long_double") => IRDataElement::LongDouble(
                value.get("value")
                    .map(| val | val.as_f64())
                    .flatten()
                    .ok_or(D::Error::custom("unable to deserialize IR floating-point data value"))?),
            Some("string") => IRDataElement::String(
                value.get("content")
                    .map(| val | val.as_str())
                    .flatten()
                    .ok_or(D::Error::custom("unable to deserialize IR string data value"))?.to_owned().into_bytes()),
            Some("pointer") => IRDataElement::Pointer{
                base: value.get("reference")
                    .map(| val | val.as_str())
                    .flatten()
                    .ok_or(D::Error::custom("unable to deserialize IR pointer data value"))?.to_owned(),
                offset: value.get("offset")
                    .map(| val | val.as_i64())
                    .flatten()
                    .ok_or(D::Error::custom("unable to deserialize IR pointer data value"))?
            },
            Some("string_pointer") => IRDataElement::StringPointer{
                base: value.get("string")
                    .map(| val | val.as_u64())
                    .flatten()
                    .ok_or(D::Error::custom("unable to deserialize IR pointer data value"))?,
                offset: value.get("offset")
                    .map(| val | val.as_i64())
                    .flatten()
                    .ok_or(D::Error::custom("unable to deserialize IR pointer data value"))?
            },
            Some("raw") => {
                match value.get("value").map(| val | val.as_array()).flatten() {
                    Some(arr) => IRDataElement::Raw(arr
                        .iter()
                        .map(| x | x
                            .as_u64()
                            .map(| e | e as u8)
                            .ok_or(D::Error::custom("unable to deserialize IR raw data")))
                        .collect::<Result<Vec<u8>, D::Error>>()?),
                    None => Err(D::Error::custom("unable to deserialize IR raw data"))?
                }
            },
            _ => Err(D::Error::custom("unable to deserialize IR data element"))?
        };
        Ok(data_entry)
    }
}

impl<'de> Deserialize<'de> for IRData {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let identifier = value.get("identifier")
            .map(| val | val.as_str())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR data identifier"))?.to_owned();
        let storage = match value.get("storage").map(| val | val.as_str()).flatten() {
            Some("global") => IRDataStorage::Global,
            Some("thread_local") => IRDataStorage::ThreadLocal,
            _ => Err(D::Error::custom("unable to deserialize IR data storage specifier"))?
        };
        let type_id = value.get("type")
            .map(| val | val.as_u64())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR data type"))?;
        let data: Result<Vec<IRDataElement>, _> = match value.get("value").map(| val | val.as_array()).flatten() {
            Some(x) => x.iter().map(| x | IRDataElement::deserialize(x.clone().into_deserializer()).map_err(D::Error::custom)).collect(),
            None => Err(D::Error::custom("unable to deserialize IR data"))
        };
        Ok(IRData::new(identifier, storage, type_id, data?))
    }
}

impl<'de> Deserialize<'de> for IRStringLiteral {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let identifier = value.get("id")
            .map(| val | val.as_u64())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR string literal"))?;
        let public = value.get("public")
            .map(| val | val.as_bool())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR string literal"))?;
        let content = match value.get("type").map(| val | val.as_str()).flatten() {
            Some("multibyte") =>
                IRStringLiteralContent::Multibyte(value.get("literal")
                    .map(| val | val.as_str())
                    .flatten()
                    .map(| x | x.to_owned().into_bytes())
                    .ok_or(D::Error::custom("unable to deserialize IR string literal content"))?),
            Some("unicode16") =>
                IRStringLiteralContent::Unicode16(value.get("literal")
                    .map(| val | val.as_array())
                    .flatten()
                    .ok_or(D::Error::custom("unable to deserialize IR string literal content"))?
                    .iter()
                    .map(| elem: &serde_json::Value |
                        elem
                            .as_u64()
                            .map(| x | x as u16)
                            .ok_or(D::Error::custom("unable to deserialize IR string literal content")))
                    .collect::<Result<Vec<u16>, D::Error>>()?),
            Some("unicode32") =>
                IRStringLiteralContent::Unicode32(value.get("literal")
                    .map(| val | val.as_array())
                    .flatten()
                    .ok_or(D::Error::custom("unable to deserialize IR string literal content"))?
                    .iter()
                    .map(| elem: &serde_json::Value |
                        elem
                            .as_u64()
                            .map(| x | x as u32)
                            .ok_or(D::Error::custom("unable to deserialize IR string literal content")))
                    .collect::<Result<Vec<u32>, D::Error>>()?),
            _ => Err(D::Error::custom("unable to deserialize IR string literal content"))?
        };

        Ok(IRStringLiteral::new(identifier, public, content))
    }
}

impl<'de> IRInlineAssembly {

    fn deserialize_param_class<D>(param_value: &serde_json::Value, type_field: &str, type_index_field: &str, index_field: &str) -> Result<(IRTypeRef, u64), D::Error>
        where D: Deserializer<'de> {
        let type_id = param_value.get(type_field)
            .map(| val | val.as_u64())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR inline assembly class"))?;
        let type_index = param_value.get(type_index_field)
            .map(| val | val.as_u64())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR inline assembly class"))? as usize;
        let index = param_value.get(index_field)
            .map(| val | val.as_u64())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR inline assembly class"))?;
        Ok((IRTypeRef::new(type_id, type_index), index))
    }
}

impl<'de> Deserialize<'de> for IRInlineAssembly {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let identifier = value.get("identifier")
            .map(| val | val.as_u64())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR inline assembly identifier"))?;
        let global = value.get("global")
            .map(| val | val.as_bool())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR inline assembly properties"))?;
        let template = value.get("template")
            .map(| val | val.as_str())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR inline assembly template"))?.to_owned();
        let parameters = value.get("parameters")
            .map(| val | val.as_array())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR inline assembly parameters"))?
            .iter()
            .map(| param_value | -> Result<(IRIdentifier, IRInlineAssemblyParameter), D::Error> {
                let param_id = param_value.get("identifier")
                    .map(| val | val.as_u64())
                    .flatten()
                    .ok_or(D::Error::custom("unable to deserialize IR inline assembly parameter identifier"))?;
                let param_aliases = param_value.get("names")
                    .map(| val | val.as_array())
                    .flatten()
                    .ok_or(D::Error::custom("unable to deserialize IR inline assembly parameter aliases"))?
                    .iter()
                    .map(| alias_value | -> Result<String, D::Error>{
                        alias_value
                            .as_str()
                            .map(| s | s.to_owned())
                            .ok_or(D::Error::custom("unable to deserialize IR inline assembly parameter aliases"))
                    })
                    .collect::<Result<Vec<String>, D::Error>>()?;
                let param_class = match param_value.get("class").map(| val | val.as_str()).flatten() {
                    Some("read") => {
                        let (typeref, index) = IRInlineAssembly::deserialize_param_class::<D>(param_value, "type", "type_index", "from")?;
                        IRInlineAssemblyParameterClass::Read(typeref, index)
                    },

                    Some("load") => {
                        let (typeref, index) = IRInlineAssembly::deserialize_param_class::<D>(param_value, "type", "type_index", "from")?;
                        IRInlineAssemblyParameterClass::Load(typeref, index)
                    },

                    Some("store") => {
                        let (typeref, index) = IRInlineAssembly::deserialize_param_class::<D>(param_value, "type", "type_index", "to")?;
                        IRInlineAssemblyParameterClass::Store(typeref, index)
                    },

                    Some("load_store") => {
                        let (typeref, index) = IRInlineAssembly::deserialize_param_class::<D>(param_value, "type", "type_index", "from_to")?;
                        IRInlineAssemblyParameterClass::LoadStore(typeref, index)
                    },

                    Some("read_store") => {
                        let (from_typeref, from_index) = IRInlineAssembly::deserialize_param_class::<D>(param_value, "from_type", "from_type_index", "from")?;
                        let (to_typeref, to_index) = IRInlineAssembly::deserialize_param_class::<D>(param_value, "from_type", "from_type_index", "from")?;
                        IRInlineAssemblyParameterClass::ReadStore(from_typeref, from_index, to_typeref, to_index)
                    },

                    Some("immediate") => {
                        let type_id = param_value.get("type")
                            .map(| val | val.as_u64())
                            .flatten()
                            .ok_or(D::Error::custom("unable to deserialize IR inline assembly class"))?;
                        let type_index = param_value.get("type_index")
                            .map(| val | val.as_u64())
                            .flatten()
                            .ok_or(D::Error::custom("unable to deserialize IR inline assembly class"))? as usize;
                        let type_ref = IRTypeRef::new(type_id, type_index);
                        let imm_value = param_value.get("value")
                            .map(| val | val.as_i64())
                            .flatten()
                            .ok_or(D::Error::custom("unable to deserialize IR inline assembly class"))?;

                        match param_value.get("variant").map(| val | val.as_str()).flatten() {
                            Some("identifier_based") => {
                                match param_value.as_str() {
                                    Some(x) => IRInlineAssemblyParameterClass::ImmediateIdentifierBased(type_ref, x.to_owned(), imm_value),
                                    _ => IRInlineAssemblyParameterClass::ImmediateConstant(type_ref, imm_value)
                                }
                            },

                            Some("literal_based") => {
                                match param_value.as_u64() {
                                    Some(x) => IRInlineAssemblyParameterClass::ImmediateLiteralBased(type_ref, x, imm_value),
                                    _ => Err(D::Error::custom("unable to deserialize IR inline assembly class"))?
                                }
                            }

                            _ => Err(D::Error::custom("unable to deserialize IR inline assembly class"))?
                        }
                    }

                    _ => Err(D::Error::custom("unable to deserialize IR inline assembly class"))?
                };

                let param_constraint = match param_value.get("constraint").map(| val | val.as_str()).flatten() {
                    Some("none") => IRInlineAssemblyParameterConstraint::None,
                    Some("register") => IRInlineAssemblyParameterConstraint::Register,
                    Some("memory") => IRInlineAssemblyParameterConstraint::Memory,
                    Some("register_memory") => IRInlineAssemblyParameterConstraint::RegisterMemory,
                    _ => Err(D::Error::custom("unable to deserialize IR inline assembly class"))?
                };

                Ok((param_id, IRInlineAssemblyParameter::new(param_id, param_aliases, param_class, param_constraint)))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        let clobbers = value.get("clobbers")
            .map(| val | val.as_array())
            .flatten()
            .ok_or(D::Error::custom("unable to deserialize IR inline assembly clobbers"))?
            .iter()
            .map(| elem | elem
                .as_str()
                .map(| s | s.to_owned())
                .ok_or(D::Error::custom("unable to deserialize IR inline assembly clobbers")))
            .collect::<Result<HashSet<String>, D::Error>>()?;
        let jump_targets = value.get("jump_targets")
                .map(| val | val.as_array())
                .flatten()
                .ok_or(D::Error::custom("unable to deserialize IR inline assembly jump targets"))?
                .iter()
                .map(| jump_elem | -> Result<(IRIdentifier, IRInlineAssemblyJumpTarget), D::Error> {
                    let jump_identifier = jump_elem.get("identifier")
                        .map(| val | val.as_u64())
                        .flatten()
                        .ok_or(D::Error::custom("unable to deserialize IR inline assembly jump target"))?;
                    let jump_aliases = jump_elem.get("names")
                        .map(| val | val.as_array())
                        .flatten()
                        .ok_or(D::Error::custom("unable to deserialize IR inline assembly jump target"))?
                        .iter()
                        .map(| alias_value | -> Result<String, D::Error> {
                            alias_value
                                .as_str()
                                .map(| s | s.to_owned())
                                .ok_or(D::Error::custom("unable to deserialize IR inline assembly jump target"))
                        })
                        .collect::<Result<Vec<String>, D::Error>>()?;
                    let jump_function = jump_elem.get("function")
                        .map(| val | val.as_str())
                        .flatten()
                        .ok_or(D::Error::custom("unable to deserialize IR inline assembly jump target"))?
                        .to_owned();
                    let jump_target = jump_elem.get("target")
                        .map(| val | val.as_u64())
                        .flatten()
                        .ok_or(D::Error::custom("unable to deserialize IR inline assembly jump target"))? as usize;
                    Ok((jump_identifier, IRInlineAssemblyJumpTarget::new(jump_identifier, jump_aliases, jump_function, jump_target)))
                })
                .collect::<Result<HashMap<IRIdentifier, IRInlineAssemblyJumpTarget>, D::Error>>()?;

        IRInlineAssembly::new(identifier, global, template, parameters, clobbers, jump_targets)
            .map_err(D::Error::custom)
    }
}

impl<'de> IRModule {
    fn deserialize_array<D, E>(value: &serde_json::Value, err: E) -> Result<Vec<D>, E>
    where
        D: Deserialize<'de>,
        E: Error {
        match value.as_array() {
            Some(arr) => arr.iter().map(| x | D::deserialize(x.clone().into_deserializer()).map_err(E::custom)).collect(),
            None => Err(err)
        }
    }
}


impl<'de> Deserialize<'de> for IRModule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let globals = IRModule::deserialize_array::<IRSymbol, D::Error>(
            value.get("globals").ok_or(D::Error::custom("unable to deserialize IR module globals"))?,
            D::Error::custom("unable to deserialize IR module globals"))?
            .into_iter()
            .map(| sym | (sym.name().to_owned(), sym))
            .collect::<HashMap<String, _>>();
        let externals = IRModule::deserialize_array::<IRSymbol, D::Error>(
            value.get("externals").ok_or( D::Error::custom("unable to deserialize IR module externals"))?,
            D::Error::custom("unable to deserialize IR module externals"))?
            .into_iter()
            .map(| sym | (sym.name().to_owned(), sym))
            .collect::<HashMap<String, _>>();
        let types = IRModule::deserialize_array::<IRType, D::Error>(
            value.get("types").ok_or(D::Error::custom("unable to deserialize IR module types"))?,
            D::Error::custom("unable to deserialize IR module types"))?
            .into_iter()
            .map(| tp | (tp.identifier(), tp))
            .collect::<HashMap<IRIdentifier, _>>();
        let decls = IRModule::deserialize_array::<IRFunctionDeclaration, D::Error>(
            value.get("function_declarations").ok_or(D::Error::custom("unable to deserialize IR module function declarations"))?,
            D::Error::custom("unable to deserialize IR module function declarations"))?
            .into_iter()
            .map(| tp | (tp.identifier(), tp))
            .collect::<HashMap<IRIdentifier, _>>();
        let funcs = IRModule::deserialize_array::<IRFunction, D::Error>(
            value.get("functions").ok_or(D::Error::custom("unable to deserialize IR module functions"))?,
            D::Error::custom("unable to deserialize IR module functions"))?
            .into_iter()
            .map(| sym | (sym.name().to_owned(), sym))
            .collect::<HashMap<String, _>>();
        let data = IRModule::deserialize_array::<IRData, D::Error>(
        value.get("data").ok_or(D::Error::custom("unable to deserialize IR module data"))?,
         D::Error::custom("unable to deserialize IR module data"))?
            .into_iter()
            .map(| tp | (tp.name().to_owned(), tp))
            .collect::<HashMap<String, _>>();
        let string_literals = IRModule::deserialize_array::<IRStringLiteral, D::Error>(
            value.get("string_literals").ok_or(D::Error::custom("unable to deserialize IR module string literals"))?,
            D::Error::custom("unable to deserialize IR module string literals"))?
            .into_iter()
            .map(| elem | (elem.identifier(), elem))
            .collect::<HashMap<IRIdentifier, _>>();
        let inline_assembly = IRModule::deserialize_array::<IRInlineAssembly, D::Error>(
            value.get("inline_assembly").ok_or(D::Error::custom("unable to deserialize IR module inline assembly"))?,
            D::Error::custom("unable to deserialize IR module inline assembly"))?
            .into_iter()
            .map(| elem | (elem.identifier(), elem))
            .collect::<HashMap<IRIdentifier, _>>();
        IRModule::new(globals, externals, types, string_literals, decls, funcs, data, inline_assembly)
            .map_err(D::Error::custom)
    }
}
