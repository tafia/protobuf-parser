//! A nom-based protobuf file parser
//!
//! This crate can be seen as a rust transcription of the
//! [descriptor.proto](https://github.com/google/protobuf/blob/master/src/google/protobuf/descriptor.proto) file

mod parser;

use parser::Parser;
use parser::Loc;

pub use parser::ParserError;
pub use parser::ParserErrorWithLocation;

/// Protobox syntax
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

/// A field rule
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Rule {
    /// A well-formed message can have zero or one of this field (but not more than one).
    Optional,
    /// This field can be repeated any number of times (including zero) in a well-formed message.
    /// The order of the repeated values will be preserved.
    Repeated,
    /// A well-formed message must have exactly one of this field.
    Required,
}

/// Protobuf supported field types
///
/// TODO: Groups (even if deprecated)
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
    /// Protobuf message or enum (holds the name)
    MessageOrEnum(String),
    /// Protobut map
    Map(Box<(FieldType, FieldType)>),
    /// Protobuf group (deprecated)
    Group(Vec<Field>),
}

/// A Protobuf Field
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Field {
    /// Field name
    pub name: String,
    /// Field `Rule`
    pub rule: Rule,
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

/// Extension range
#[derive(Default, Debug, Eq, PartialEq, Copy, Clone)]
pub struct FieldNumberRange {
    /// First number
    pub from: i32,
    /// Inclusive
    pub to: i32,
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
    ///
    /// TODO: use RangeInclusive once stable
    pub reserved_nums: Vec<FieldNumberRange>,
    /// Message reserved names
    pub reserved_names: Vec<String>,
    /// Nested messages
    pub messages: Vec<Message>,
    /// Nested enums
    pub enums: Vec<Enumeration>,
}

/// A protobuf enumeration field
#[derive(Debug, Clone)]
pub struct EnumValue {
    /// enum value name
    pub name: String,
    /// enum value number
    pub number: i32,
}

/// A protobuf enumerator
#[derive(Debug, Clone)]
pub struct Enumeration {
    /// enum name
    pub name: String,
    /// enum values
    pub values: Vec<EnumValue>,
}

/// A OneOf
#[derive(Debug, Clone, Default)]
pub struct OneOf {
    /// OneOf name
    pub name: String,
    /// OneOf fields
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Extension {
    /// Extend this type with field
    pub extendee: String,
    /// Extension field
    pub field: Field,
}

/// A File descriptor representing a whole .proto file
#[derive(Debug, Default, Clone)]
pub struct FileDescriptor {
    /// Imports
    pub import_paths: Vec<String>,
    /// Package
    pub package: String,
    /// Protobuf Syntax
    pub syntax: Syntax,
    /// Top level messages
    pub messages: Vec<Message>,
    /// Enums
    pub enums: Vec<Enumeration>,
    /// Extensions
    pub extensions: Vec<Extension>,
}

impl FileDescriptor {
    /// Parses a .proto file content into a `FileDescriptor`
    pub fn parse<S: AsRef<str>>(file: S) -> Result<Self, ParserErrorWithLocation> {
        let mut parser = Parser::new(file.as_ref());
        match parser.next_proto() {
            Ok(r) => Ok(r),
            Err(error) => {
                let Loc { line, col } = parser.loc();
                Err(ParserErrorWithLocation { error, line, col })
            }
        }
    }
}
