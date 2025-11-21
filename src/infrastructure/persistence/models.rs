/// Data models for FondsPod - Future features only
/// Schema and SchemaItem models are defined in domain::models
/// This file contains models for features not yet implemented in the UI

use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::infrastructure::persistence::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, QueryableByName)]
#[diesel(table_name = fond_classifications)]
pub struct FondClassification {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub code: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub parent_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, QueryableByName)]
#[diesel(table_name = fonds)]
pub struct Fond {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub fond_no: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub fond_classification_code: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub created_at: String, // ISO 8601 format
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
    #[diesel(column_name = "createAt")]
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub create_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, QueryableByName)]
#[diesel(table_name = fond_schemas)]
pub struct FondSchema {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub fond_no: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub schema_no: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub order_no: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, QueryableByName)]
#[diesel(table_name = series)]
pub struct Series {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub series_no: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub fond_no: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
    #[diesel(column_name = "createAt")]
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub create_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, QueryableByName)]
#[diesel(table_name = files)]
pub struct File {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub file_no: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub series_no: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
    #[diesel(column_name = "createAt")]
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub create_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, QueryableByName)]
#[diesel(table_name = items)]
pub struct Item {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub item_no: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub file_no: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
    #[diesel(column_name = "createAt")]
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub create_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, QueryableByName)]
#[diesel(table_name = sequences)]
pub struct Sequence {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub prefix: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub current_value: i32,
}

// Note: Additional models (Fond, Series, File, etc.) are kept for future features
// but are not currently used in the UI. They will be utilized when those features
// are implemented in the new layered architecture.
