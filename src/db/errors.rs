use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Error {
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
    NotEnoughTextBytes { len: usize, bytes_read: usize },
}
