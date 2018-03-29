#[macro_use] extern crate nom;

mod parser;
mod parser_with_dependencies;

use std::path::PathBuf;
use parser::file_descriptor;


pub use parser_with_dependencies::parse_with_dependencies;
pub use parser_with_dependencies::FileDescriptorWithContext;


#[derive(Debug, Clone, Copy)]
pub enum Syntax {
    Proto2,
    Proto3,
}

impl Default for Syntax {
    fn default() -> Syntax {
        Syntax::Proto2
    }
}

#[derive(Debug, Clone)]
pub enum Frequency {
    Optional,
    Repeated,
    Required,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldType {
    Int32,
    Int64,
    Uint32,
    Uint64,
    Sint32,
    Sint64,
    Bool,
    Enum(String),
    Fixed64,
    Sfixed64,
    Double,
    String_,
    Bytes,
    Message(String),
    Fixed32,
    Sfixed32,
    Float,
    Map(Box<(FieldType, FieldType)>),
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub frequency: Frequency,
    pub typ: FieldType,
    pub number: i32,
    pub default: Option<String>,
    pub packed: Option<bool>,
    pub boxed: bool,
    pub deprecated: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Message {
    pub name: String,
    pub fields: Vec<Field>,
    pub oneofs: Vec<OneOf>,
    pub reserved_nums: Option<Vec<i32>>,
    pub reserved_names: Option<Vec<String>>,
    pub imported: bool,
    pub package: String,        // package from imports + nested items
    pub messages: Vec<Message>, // nested messages
    pub enums: Vec<Enumerator>, // nested enums
    pub module: String,         // 'package' corresponding to actual generated Rust module
}

#[derive(Debug, Clone)]
pub struct Enumerator {
    pub name: String,
    pub fields: Vec<(String, i32)>,
    pub imported: bool,
    pub package: String,
    pub module: String,
}

#[derive(Debug, Clone, Default)]
pub struct OneOf {
    pub name: String,
    pub fields: Vec<Field>,
    pub package: String,
    pub module: String,
    pub imported: bool,
}

#[derive(Debug, Default, Clone)]
pub struct FileDescriptor {
    pub import_paths: Vec<PathBuf>,
    pub package: String,
    pub syntax: Syntax,
    pub messages: Vec<Message>,
    pub enums: Vec<Enumerator>,
    pub module: String,
}

impl FileDescriptor {
    /// Parses a .proto file content into a `FileDescriptor`
    pub fn parse<S: AsRef<[u8]>>(file: S) -> Result<Self, ::nom::IError> {
        file_descriptor(file.as_ref()).to_full_result()
    }
}
