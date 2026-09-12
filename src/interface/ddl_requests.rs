use serde::{Deserialize, Serialize};

use super::{DatabaseId, SchemaId, TableId, TableSchema};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DDLRequest {
    // Create
    CreateDB(DatabaseId),
    CreateSchema(SchemaId),
    CreateTable(TableId, TableSchema),
    // Drop
    DropDB(DatabaseId),
    DropSchema(SchemaId),
    DropTable(TableId),
}
