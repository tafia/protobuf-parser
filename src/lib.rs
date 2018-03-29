#[macro_use]
extern crate nom;

mod parser;

use std::path::PathBuf;
use parser::file_descriptor;

/// Protobox syntax
#[derive(Debug, Clone, Copy)]
pub enum Syntax {
    /// Protobuf syntax [2](https://developers.google.com/protocol-buffers/docs/proto) (default)
    Proto2,
    /// Protobuf syntax [3](https://developers.google.com/protocol-buffers/docs/proto3)
    Proto3,
}

impl Default for Syntax {
    fn default() -> Syntax {
        Syntax::Proto2
    }
}

/// A field frequency
#[derive(Debug, Clone)]
pub enum Frequency {
    /// A well-formed message can have zero or one of this field (but not more than one).
    Optional,
    /// This field can be repeated any number of times (including zero) in a well-formed message.
    /// The order of the repeated values will be preserved.
    Repeated,
    /// A well-formed message must have exactly one of this field.
    Required,
}

/// Protobuf supported field types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldType {
    /// Protobuf int32
    ///
    /// # Remarks
    ///
    /// Uses variable-length encoding. Inefficient for encoding negative numbers – if
    /// your field is likely to have negative values, use sint32 instead.
    Int32,
    /// Protobuf int64
    ///
    /// # Remarks
    ///
    /// Uses variable-length encoding. Inefficient for encoding negative numbers – if
    /// your field is likely to have negative values, use sint64 instead.
    Int64,
    /// Protobuf uint32
    ///
    /// # Remarks
    ///
    /// Uses variable-length encoding.
    Uint32,
    /// Protobuf uint64
    ///
    /// # Remarks
    ///
    /// Uses variable-length encoding.
    Uint64,
    /// Protobuf sint32
    ///
    /// # Remarks
    ///
    /// Uses ZigZag variable-length encoding. Signed int value. These more efficiently
    /// encode negative numbers than regular int32s.
    Sint32,
    /// Protobuf sint64
    ///
    /// # Remarks
    ///
    /// Uses ZigZag variable-length encoding. Signed int value. These more efficiently
    /// encode negative numbers than regular int32s.
    Sint64,
    /// Protobuf bool
    Bool,
    /// Protobuf enum (holds the enum name)
    Enum(String),
    /// Protobuf fixed64
    ///
    /// # Remarks
    ///
    /// Always eight bytes. More efficient than uint64 if values are often greater than 2^56.
    Fixed64,
    /// Protobuf sfixed64
    ///
    /// # Remarks
    ///
    /// Always eight bytes.
    Sfixed64,
    /// Protobuf double
    Double,
    /// Protobuf string
    ///
    /// # Remarks
    ///
    /// A string must always contain UTF-8 encoded or 7-bit ASCII text.
    String,
    /// Protobuf bytes
    ///
    /// # Remarks
    ///
    /// May contain any arbitrary sequence of bytes.
    Bytes,
    /// Protobut message (holds the message name)
    Message(String),
    /// Protobut fixed32
    ///
    /// # Remarks
    ///
    /// Always four bytes. More efficient than uint32 if values are often greater than 2^28.
    Fixed32,
    /// Protobut sfixed32
    ///
    /// # Remarks
    ///
    /// Always four bytes.
    Sfixed32,
    /// Protobut float
    Float,
    /// Protobut map
    Map(Box<(FieldType, FieldType)>),
}

/// A Protobuf Field
#[derive(Debug, Clone)]
pub struct Field {
    /// Field name
    pub name: String,
    /// Field `Frequency`
    pub frequency: Frequency,
    /// Field type
    pub typ: FieldType,
    /// Tag number
    pub number: i32,
    /// Default value for the field
    pub default: Option<String>,
    /// Packed property for repeated fields
    pub packed: Option<bool>,
    /// Is the field deprecated
    pub deprecated: bool,
}

/// A protobuf message
#[derive(Debug, Clone, Default)]
pub struct Message {
    /// Message name
    pub name: String,
    /// Message `Field`s
    pub fields: Vec<Field>,
    /// Message `OneOf`s
    pub oneofs: Vec<OneOf>,
    /// Message reserved numbers
    pub reserved_nums: Option<Vec<i32>>,
    /// Message reserved names
    pub reserved_names: Option<Vec<String>>,
    /// Nested messages
    pub messages: Vec<Message>,
    /// Nested enums
    pub enums: Vec<Enumerator>,
}

/// A protobuf enumerator field
#[derive(Debug, Clone)]
pub struct EnumVariant {
    /// enum variant name
    pub name: String,
    /// enum variant number
    pub number: i32,
}

/// A protobuf enumerator
#[derive(Debug, Clone)]
pub struct Enumerator {
    /// enum name
    pub name: String,
    /// enum variants
    pub variants: Vec<EnumVariant>,
}

/// A OneOf
#[derive(Debug, Clone, Default)]
pub struct OneOf {
    /// OneOf name
    pub name: String,
    /// OneOf fields
    pub fields: Vec<Field>,
}

/// A File descriptor representing a whole .proto file
#[derive(Debug, Default, Clone)]
pub struct FileDescriptor {
    /// Imports
    pub import_paths: Vec<PathBuf>,
    /// Package
    pub package: String,
    /// Protobuf Syntax
    pub syntax: Syntax,
    /// Top level messages
    pub messages: Vec<Message>,
    /// Enums
    pub enums: Vec<Enumerator>,
}

impl FileDescriptor {
    /// Parses a .proto file content into a `FileDescriptor`
    pub fn parse<S: AsRef<[u8]>>(file: S) -> Result<Self, ::nom::IError> {
        file_descriptor(file.as_ref()).to_full_result()
    }
}
