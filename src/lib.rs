use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub a: String,
    pub b: i32,
    pub c: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestType {
    Close,
    Select,
    Insert,
    Update,
    Delete,
}
