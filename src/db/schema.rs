use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Integer,
    Float,
    Text,
    Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub nullable: bool,
    pub data_type: DataType,
}

pub type Schema = Vec<ColumnDefinition>;
