use std::io::BufRead;
use std::io::Read;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ColumnDefinition;
use super::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Datum {
    Null,
    Integer(i64),
    Float(f64),
    Text(String),
    Uuid(Uuid),
}

impl Datum {}

pub type Tuple = Vec<Datum>;
