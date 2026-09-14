use serde::{Deserialize, Serialize};

use crate::db::DataType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseId {
    db_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaId {
    pub db_name: String,
    pub schema_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableId {
    pub db_name: String,
    pub schema_name: String,
    pub table_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub columns: Vec<ColumnSchema>,
    pub primary_key: String,
    pub foreign_keys: Vec<ForeignKey>,
    pub unique_constraints: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSchema {
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKey {
    pub column_name: String,
    pub ref_table: TableId,
    pub ref_column: String,
}
