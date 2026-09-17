use std::{fmt::Display, hash::Hash};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
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
}

impl Display for PageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.db_name, self.schema_name, self.collection_name, self.page_no
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_id_to_string() {
        let page_id = PageId::new(
            "MY_DB".to_string(),
            "MY_SCHEMA".to_string(),
            "MY_TABLE".to_string(),
            2,
        );

        assert_eq!(page_id.to_string(), "MY_DB.MY_SCHEMA.MY_TABLE.2")
    }
}
