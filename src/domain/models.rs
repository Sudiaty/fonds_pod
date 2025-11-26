/// Domain models representing core business entities
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = crate::infrastructure::persistence::schema::schemas)]
pub struct Schema {
    pub schema_no: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = crate::infrastructure::persistence::schema::schema_items)]
pub struct SchemaItem {
    pub schema_no: String,
    pub item_no: String,
    pub item_name: String,
}

#[derive(Debug, Clone)]
pub struct ArchiveLibrary {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub language: String,
    pub archive_libraries: Vec<ArchiveLibrary>,
    pub last_opened_library: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            language: "zh_CN".to_string(),
            archive_libraries: Vec::new(),
            last_opened_library: None,
        }
    }
}

// Business rule constants
pub const PROTECTED_SCHEMA: &str = "Year";

/// Validates if a schema operation is allowed
pub fn can_modify_schema(schema_no: &str) -> bool {
    schema_no != PROTECTED_SCHEMA
}
