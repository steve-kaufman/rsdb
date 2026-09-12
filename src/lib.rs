pub mod ddl;
pub mod ddl_requests;

pub use ddl::*;
pub use ddl_requests::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Close,
    DDLRequest(DDLRequest),
    DMLRequest {
        transaction: Uuid,
        request: DMLRequest,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DMLRequest {
    Select(SelectRequest),
    Insert(InsertRequest),
    Update(UpdateRequest),
    Delete(DeleteRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectRequest {
    pub transaction: Transaction,
    pub from: FromExpr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub create_time: u64,
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FromExpr {
    TableId(TableId),
}
