use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageId {
    pub db_name: String,
    pub schema_name: String,
    pub collection_name: String,
    pub page_no: usize,
}

impl PageId {
    pub fn new(
        db_name: String,
        schema_name: String,
        collection_name: String,
        page_no: usize,
    ) -> Self {
        Self {
            db_name,
            schema_name,
            collection_name,
            page_no,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            self.db_name, self.schema_name, self.collection_name, self.page_no
        )
    }
}
