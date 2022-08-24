use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use serde::Deserialize;
use quick_xml;

#[derive(Debug, Deserialize)]
enum OpcodeClass {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "coderef")]
    CodeReference,
    #[serde(rename = "funcref")]
    FunctionReference,
    #[serde(rename = "typeref")]
    TypeReference,
    #[serde(rename = "i64")]
    Integer64,
    #[serde(rename = "u64")]
    UInteger64,
    #[serde(rename = "u32")]
    UInteger32,
    #[serde(rename = "f64")]
    Float64,
    #[serde(rename = "f32")]
    Float32,
    #[serde(rename = "bool")]
    Boolean,
    #[serde(rename = "string")]
    String,
    #[serde(rename = "identifier")]
    Identifier,
    #[serde(rename = "memflags")]
    MemFlags
}

#[derive(Debug, Deserialize)]
struct Opcode {
    #[serde(rename = "id")]
    identifier: String,
    mnemonic: String,
    code: String,
    #[serde(rename = "type")]
    klass: OpcodeClass
}

#[derive(Debug, Deserialize)]
struct Opcodes {
    revision: Option<String>,
    #[serde(rename = "opcode")]
    opcodes: Vec<Opcode>
}

fn gen_opcodes(input_path: &Path, out_dir: &OsString) {
    let dest_path = Path::new(&out_dir).join("opcodes.rs");

    let opcodes_reader = io::BufReader::new(
        fs::File::open(input_path).unwrap());
    let opcodes: Opcodes = quick_xml::de::from_reader(opcodes_reader).unwrap();

    let mut output_writer = fs::File::create(dest_path).unwrap();
    write!(&mut output_writer, "#[allow(non_camel_case_types)]\n").unwrap();
    write!(&mut output_writer, "#[derive(Debug)]\n").unwrap();
    write!(&mut output_writer, "pub enum IRInstruction {{\n").unwrap();
    for opcode in opcodes.opcodes.iter() {
        write!(&mut output_writer, "    {}", opcode.identifier).unwrap();
        match opcode.klass {
            OpcodeClass::None => (),
            OpcodeClass::Integer64 => write!(&mut output_writer, "(i64)").unwrap(),
            OpcodeClass::UInteger64 | OpcodeClass::String
                => write!(&mut output_writer, "(u64)").unwrap(),
            OpcodeClass::CodeReference
                => write!(&mut output_writer, "(usize)").unwrap(),
            OpcodeClass::Identifier => write!(&mut output_writer, "(String)").unwrap(),
            OpcodeClass::FunctionReference => write!(&mut output_writer, "(u64, Option<String>)").unwrap(),
            OpcodeClass::UInteger32 => write!(&mut output_writer, "(u32, u32)").unwrap(),
            OpcodeClass::TypeReference => write!(&mut output_writer, "(IRTypeRef)").unwrap(),
            OpcodeClass::Float32 => write!(&mut output_writer, "(f32)").unwrap(),
            OpcodeClass::Float64 => write!(&mut output_writer, "(f64)").unwrap(),
            OpcodeClass::Boolean => write!(&mut output_writer, "(bool)").unwrap(),
            OpcodeClass::MemFlags => write!(&mut output_writer, "(IRInstructionMemFlags)").unwrap()
        };
        write!(&mut output_writer, ",\n").unwrap();
    }
    write!(&mut output_writer, "}}\n\n").unwrap();

    write!(&mut output_writer, "impl IRInstruction {{\n").unwrap();
    write!(&mut output_writer, "    pub fn code(&self) -> u64 {{\n").unwrap();
    write!(&mut output_writer, "        match self {{\n").unwrap();
    for opcode in opcodes.opcodes.iter() {
        write!(&mut output_writer, "            IRInstruction::{}", opcode.identifier).unwrap();
        match opcode.klass {
            OpcodeClass::None => (),
            OpcodeClass::UInteger64 | OpcodeClass::CodeReference | 
            OpcodeClass::Identifier | OpcodeClass::String | OpcodeClass::Integer64 |
            OpcodeClass::TypeReference | OpcodeClass::Float32 | OpcodeClass::Float64 |
            OpcodeClass::Boolean | OpcodeClass::MemFlags
                => write!(&mut output_writer, "(_)").unwrap(),
            OpcodeClass::UInteger32 | OpcodeClass::FunctionReference
              => write!(&mut output_writer, "(_, _)").unwrap(),
        };
        write!(&mut output_writer, " => {},\n", opcode.code).unwrap();
    }
    write!(&mut output_writer, "        }}\n").unwrap();
    write!(&mut output_writer, "    }}\n\n").unwrap();
    write!(&mut output_writer, "    pub fn mnemonic(&self) -> &'static str {{\n").unwrap();
    write!(&mut output_writer, "        match self {{\n").unwrap();
    for opcode in opcodes.opcodes.iter() {
        write!(&mut output_writer, "            IRInstruction::{}", opcode.identifier).unwrap();
        match opcode.klass {
            OpcodeClass::None => (),
            OpcodeClass::UInteger64 | OpcodeClass::CodeReference |
            OpcodeClass::Identifier | OpcodeClass::String | OpcodeClass::Integer64 |
            OpcodeClass::TypeReference | OpcodeClass::Float32 | OpcodeClass::Float64 |
            OpcodeClass::Boolean | OpcodeClass::MemFlags
                => write!(&mut output_writer, "(_)").unwrap(),
            OpcodeClass::UInteger32 | OpcodeClass::FunctionReference
                => write!(&mut output_writer, "(_, _)").unwrap(),
        };
        write!(&mut output_writer, " => \"{}\",\n", opcode.mnemonic).unwrap();
    }
    write!(&mut output_writer, "        }}\n").unwrap();
    write!(&mut output_writer, "    }}\n\n").unwrap();
    write!(&mut output_writer, "    pub fn argument<'a>(&'a self) -> IRInstructionArgument<'a> {{\n").unwrap();
    write!(&mut output_writer, "        match self {{\n").unwrap();
    for opcode in opcodes.opcodes.iter() {
        write!(&mut output_writer, "            IRInstruction::{}", opcode.identifier).unwrap();
        match opcode.klass {
            OpcodeClass::None => write!(&mut output_writer, " => IRInstructionArgument::None,\n").unwrap(),
            OpcodeClass::Integer64 => write!(&mut output_writer, "(x) => IRInstructionArgument::Integer(*x),\n").unwrap(),
            OpcodeClass::UInteger64 => write!(&mut output_writer, "(x) => IRInstructionArgument::UInteger(*x),\n").unwrap(),
            OpcodeClass::UInteger32 => write!(&mut output_writer, "(x, y) => IRInstructionArgument::UIntegerPair(*x, *y),\n").unwrap(),
            OpcodeClass::Float64 => write!(&mut output_writer, "(x) => IRInstructionArgument::Float64(*x),\n").unwrap(),
            OpcodeClass::Float32 => write!(&mut output_writer, "(x) => IRInstructionArgument::Float32(*x),\n").unwrap(),
            OpcodeClass::Boolean => write!(&mut output_writer, "(x) => IRInstructionArgument::Boolean(*x),\n").unwrap(),
            OpcodeClass::String => write!(&mut output_writer, "(x) => IRInstructionArgument::String(*x),\n").unwrap(),
            OpcodeClass::Identifier => write!(&mut output_writer, "(id) => IRInstructionArgument::Identifier(id),\n").unwrap(),
            OpcodeClass::TypeReference => write!(&mut output_writer, "(x) => IRInstructionArgument::TypeRef(*x),\n").unwrap(),
            OpcodeClass::CodeReference => write!(&mut output_writer, "(x) => IRInstructionArgument::CodeRef(*x),\n").unwrap(),
            OpcodeClass::FunctionReference => write!(&mut output_writer, "(id, name) => IRInstructionArgument::FunctionRef(*id, name.as_ref()),\n").unwrap(),
            OpcodeClass::MemFlags => write!(&mut output_writer, "(memflags) => IRInstructionArgument::MemFlags(*memflags),\n").unwrap(),
        };
    }
    write!(&mut output_writer, "        }}\n").unwrap();
    write!(&mut output_writer, "    }}\n\n").unwrap();
    write!(&mut output_writer, "    pub fn revision() -> Option<u64> {{\n").unwrap();
    match opcodes.revision {
        Some(rev) => write!(&mut output_writer, "        Some({})\n", rev).unwrap(),
        None => write!(&mut output_writer, "        None\n").unwrap(),
    };
    write!(&mut output_writer, "    }}\n").unwrap();
    write!(&mut output_writer, "}}").unwrap();

}

fn gen_instr_loader(input_path: &Path, out_dir: &OsString) {
    let dest_path = Path::new(&out_dir).join("instr-loader.rs");

    let opcodes_reader = io::BufReader::new(
        fs::File::open(input_path).unwrap());
    let opcodes: Opcodes = quick_xml::de::from_reader(opcodes_reader).unwrap();

    let mut output_writer = fs::File::create(dest_path).unwrap();
    write!(&mut output_writer, "fn deserialize_instruction<'de, D>(opcode_sym: &str, value: &Value) -> Result<IRInstruction, D::Error> where D: Deserializer<'de> {{\n").unwrap();
    write!(&mut output_writer, "    match opcode_sym {{\n").unwrap();
    for opcode in opcodes.opcodes.iter() {
        write!(&mut output_writer, "        \"{}\" => Ok(", opcode.mnemonic).unwrap();
        match opcode.klass {
            OpcodeClass::None 
                => write!(&mut output_writer, "IRInstruction::{}", opcode.identifier).unwrap(),
            OpcodeClass::Integer64
                => write!(&mut output_writer, "IRInstruction::{}(deserialize_instr_i64::<D>(value)?)", opcode.identifier).unwrap(),
            OpcodeClass::UInteger64 | OpcodeClass::String
                => write!(&mut output_writer, "IRInstruction::{}(deserialize_instr_u64::<D>(value)?)", opcode.identifier).unwrap(),
            OpcodeClass::CodeReference
                => write!(&mut output_writer, "IRInstruction::{}(deserialize_instr_usize::<D>(value)?)", opcode.identifier).unwrap(),
            OpcodeClass::UInteger32
                => write!(&mut output_writer, "{{ let (x, y) = deserialize_instr_u32::<D>(value)?; IRInstruction::{}(x, y) }}", opcode.identifier).unwrap(),
            OpcodeClass::FunctionReference
                => write!(&mut output_writer, "{{ let (x, y) = deserialize_instr_funcref::<D>(value)?; IRInstruction::{}(x, y) }}", opcode.identifier).unwrap(),
            OpcodeClass::Identifier
                => write!(&mut output_writer, "IRInstruction::{}(deserialize_instr_identifier::<D>(value)?)", opcode.identifier).unwrap(),
            OpcodeClass::TypeReference
                => write!(&mut output_writer, "IRInstruction::{}(deserialize_instr_typeref::<D>(value)?)", opcode.identifier).unwrap(),
            OpcodeClass::Float32
                => write!(&mut output_writer, "IRInstruction::{}(deserialize_instr_f32::<D>(value)?)", opcode.identifier).unwrap(),
            OpcodeClass::Float64
            => write!(&mut output_writer, "IRInstruction::{}(deserialize_instr_f64::<D>(value)?)", opcode.identifier).unwrap(),
            OpcodeClass::Boolean
                => write!(&mut output_writer, "IRInstruction::{}(deserialize_instr_bool::<D>(value)?)", opcode.identifier).unwrap(),
            OpcodeClass::MemFlags
                => write!(&mut output_writer, "IRInstruction::{}(deserialize_instr_memflags::<D>(value)?)", opcode.identifier).unwrap(),
        };
        write!(&mut output_writer, "),\n").unwrap();
    }
    write!(&mut output_writer, "        &_ => Err(D::Error::custom(\"Unable to deserialize instruction\".to_owned())).unwrap()\n").unwrap();
    write!(&mut output_writer, "    }}\n").unwrap();
    write!(&mut output_writer, "}}").unwrap();
}

fn main() {
    let opcodes_definition_file = "kefir/resources/opcodes.xml";
    let input_path = Path::new(opcodes_definition_file);
    let out_dir = env::var_os("OUT_DIR").unwrap();

    gen_opcodes(input_path, &out_dir);
    gen_instr_loader(input_path, &out_dir);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", opcodes_definition_file);
}