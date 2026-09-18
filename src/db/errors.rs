use serde::{Deserialize, Serialize};

use crate::db::{ColumnDefinition, Datum};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Error {
    // Arbitrary Error
    Other(String),
    // Error during I/O operation, e.g. read
    IO(String),
    // General error during deserialization
    DeserializeError(String),
    // General error during serialization
    SerializeError(String),
    // A null check byte was found to be something other than 0 (null) or 1 (not null)
    InvalidNullCheckByte(u8),
    // Found invalid UTF-8 bytes while parsing text
    InvalidUtf8(String),
    // Tagged text length was greater than number of bytes available
    NotEnoughTextBytes {
        len: usize,
        bytes_read: usize,
    },
    // Attempted to serialize text with length greater than u16::MAX (65,535).
    // Contains length of unserializable text
    TextOverflow(usize),
    // Cannot serialize an empty tuple
    SerializeEmptyTuple,
    // Tuples must have same number of items as schema
    TupleSchemaLenMismatch {
        tuple_len: usize,
        schema_len: usize,
    },
    // Tuple datums must have a compatible data type with the corresponding column definition
    TupleSchemaTypeMismatch {
        column_definition: ColumnDefinition,
        datum: Datum,
    },
    // Tried to write a null to a non-nullable column
    NullMismatch {
        column_definition: ColumnDefinition,
    },
    // Tried to deserialize with an empty schema
    DeserializeEmptySchema,
    // Tried to write an incorrect number of bytes as a page
    InvalidPageSize(usize),
}

impl Error {
    pub fn tuple_schema_type_mismatch(column_definition: &ColumnDefinition, datum: &Datum) -> Self {
        Self::TupleSchemaTypeMismatch {
            column_definition: column_definition.clone(),
            datum: datum.clone(),
        }
    }
}
