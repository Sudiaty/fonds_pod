/// Schema service - coordinates schema operations with business rules
use crate::domain::models::{Schema, SchemaItem};
use crate::domain::can_modify_schema;
use crate::infrastructure::persistence::{queries, establish_connection};
use std::error::Error;
use std::path::Path;

/// Schema service with direct database access
pub struct SchemaService;

impl SchemaService {
    /// Create a new schema
    pub fn create_schema(db_path: &Path, schema_no: String, name: String) -> Result<(), Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        let schema = Schema { schema_no, name };
        queries::create_schema(&mut conn, &schema)
    }
    
    /// List all schemas
    pub fn list_schemas(db_path: &Path) -> Result<Vec<Schema>, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        queries::list_schemas(&mut conn)
    }
    
    /// Delete schema (with business rule validation)
    pub fn delete_schema(db_path: &Path, schema_no: String) -> Result<bool, Box<dyn Error>> {
        if !can_modify_schema(&schema_no) {
            return Err(format!("Schema '{}' cannot be deleted", schema_no).into());
        }
        let mut conn = establish_connection(db_path)?;
        queries::delete_schema(&mut conn, &schema_no)
    }
    
    /// Add schema item
    pub fn add_schema_item(
        db_path: &Path,
        schema_no: String,
        item_no: String,
        item_name: String,
    ) -> Result<(), Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        let item = SchemaItem {
            schema_no,
            item_no,
            item_name,
        };
        queries::create_schema_item(&mut conn, &item)
    }
    
    /// List schema items
    pub fn list_schema_items(db_path: &Path, schema_no: &str) -> Result<Vec<SchemaItem>, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        queries::list_schema_items(&mut conn, schema_no)
    }
    
    /// Delete schema item
    pub fn delete_schema_item(
        db_path: &Path,
        schema_no: String,
        item_no: String,
    ) -> Result<bool, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        queries::delete_schema_item(&mut conn, &schema_no, &item_no)
    }
}
