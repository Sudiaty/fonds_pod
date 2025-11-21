/// Database queries for FondsPod
/// Implements CRUD operations using Diesel ORM with SQLite
/// 
/// Note: This module contains queries for current and future features.
/// Some functions are not yet used but will be when additional UI features are implemented.

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use crate::domain::models::{Schema, SchemaItem};
use crate::infrastructure::persistence::models;

// Schema operations
pub fn create_schema(
    conn: &mut SqliteConnection,
    schema: &Schema,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::schemas::dsl::*;
    
    diesel::insert_into(schemas)
        .values(schema)
        .execute(conn)?;
    Ok(())
}

pub fn list_schemas(
    conn: &mut SqliteConnection,
) -> Result<Vec<Schema>, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::schemas::dsl::*;
    
    let results = schemas
        .load::<Schema>(conn)?;
    
    Ok(results)
}

pub fn delete_schema(
    conn: &mut SqliteConnection,
    schema_number: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::schemas::dsl::*;
    use crate::infrastructure::persistence::schema::fond_schemas;
    
    if schema_number == "Year" {
        return Ok(false);
    }
    
    // Check if schema is used in any fond
    let count = fond_schemas::table
        .filter(fond_schemas::schema_no.eq(schema_number))
        .count()
        .get_result::<i64>(conn)?;
    
    if count > 0 {
        return Ok(false);
    }
    
    diesel::delete(schemas.filter(schema_no.eq(schema_number)))
        .execute(conn)?;
    
    Ok(true)
}

// SchemaItem operations
pub fn create_schema_item(
    conn: &mut SqliteConnection,
    item: &SchemaItem,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::schema_items::dsl::*;
    
    diesel::insert_into(schema_items)
        .values(item)
        .execute(conn)?;
    Ok(())
}

pub fn list_schema_items(
    conn: &mut SqliteConnection,
    schema_number: &str,
) -> Result<Vec<SchemaItem>, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::schema_items::dsl::*;
    
    let results = schema_items
        .filter(schema_no.eq(schema_number))
        .load::<SchemaItem>(conn)?;
    
    Ok(results)
}

pub fn delete_schema_item(
    conn: &mut SqliteConnection,
    schema_number: &str,
    item_number: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::schema_items::dsl::*;
    
    let deleted = diesel::delete(
        schema_items
            .filter(schema_no.eq(schema_number))
            .filter(item_no.eq(item_number))
    )
    .execute(conn)?;
    
    Ok(deleted > 0)
}

// ============================================================================
// Fond Classification operations
// ============================================================================

/// Create a top-level classification (parent_id = NULL)
pub fn create_top_classification(
    conn: &mut SqliteConnection,
    classification: &models::FondClassification,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::fond_classifications::dsl::*;
    diesel::insert_into(fond_classifications)
        .values(classification)
        .execute(conn)?;
    Ok(())
}

/// Create a child classification (parent_id references existing code)
pub fn create_child_classification(
    conn: &mut SqliteConnection,
    classification: &models::FondClassification,
) -> Result<(), Box<dyn std::error::Error>> {
    create_top_classification(conn, classification)
}

/// List all top-level classifications (parent_id IS NULL)
pub fn list_top_classifications(
    conn: &mut SqliteConnection,
) -> Result<Vec<models::FondClassification>, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::fond_classifications::dsl::*;
    let results = fond_classifications
        .filter(parent_id.is_null())
        .load::<models::FondClassification>(conn)?;
    Ok(results)
}

/// List child classifications under a parent code
pub fn list_child_classifications(
    conn: &mut SqliteConnection,
    parent_code: &str,
) -> Result<Vec<models::FondClassification>, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::fond_classifications::dsl::*;
    let results = fond_classifications
        .filter(parent_id.eq(parent_code))
        .load::<models::FondClassification>(conn)?;
    Ok(results)
}

/// Check if classification has children
fn has_children(conn: &mut SqliteConnection, parent_code: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::fond_classifications::dsl::*;
    let count = fond_classifications
        .filter(parent_id.eq(parent_code))
        .count()
        .get_result::<i64>(conn)?;
    Ok(count > 0)
}

/// Check if classification referenced by fonds
fn is_referenced_by_fonds(conn: &mut SqliteConnection, class_code: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::fonds::dsl::*;
    let count = fonds
        .filter(fond_classification_code.eq(class_code))
        .count()
        .get_result::<i64>(conn)?;
    Ok(count > 0)
}

/// Delete a classification if not referenced and has no children
pub fn delete_classification(
    conn: &mut SqliteConnection,
    class_code: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::fond_classifications::dsl::*;

    if has_children(conn, class_code)? || is_referenced_by_fonds(conn, class_code)? {
        return Ok(false);
    }

    let deleted = diesel::delete(fond_classifications.filter(code.eq(class_code)))
        .execute(conn)?;
    Ok(deleted > 0)
}

/// Activate a classification
pub fn activate_classification(
    conn: &mut SqliteConnection,
    class_code: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::fond_classifications::dsl::*;
    diesel::update(fond_classifications.filter(code.eq(class_code)))
        .set(is_active.eq(true))
        .execute(conn)?;
    Ok(())
}

/// Deactivate a classification
pub fn deactivate_classification(
    conn: &mut SqliteConnection,
    class_code: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::fond_classifications::dsl::*;
    diesel::update(fond_classifications.filter(code.eq(class_code)))
        .set(is_active.eq(false))
        .execute(conn)?;
    Ok(())
}

// The following operations are kept for future features (Fond, Series, File management)
// They will be migrated to the new service layer when those features are implemented.

// Fond operations
pub fn create_fond(
    conn: &mut SqliteConnection,
    fond: &models::Fond,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::fonds::dsl::*;
    
    diesel::insert_into(fonds)
        .values(fond)
        .execute(conn)?;
    Ok(())
}

pub fn get_fond(
    conn: &mut SqliteConnection,
    fond_number: &str,
) -> Result<Option<models::Fond>, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::fonds::dsl::*;
    
    let result = fonds
        .filter(fond_no.eq(fond_number))
        .first::<models::Fond>(conn)
        .optional()?;
    
    Ok(result)
}

pub fn list_fonds(
    conn: &mut SqliteConnection,
) -> Result<Vec<models::Fond>, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::fonds::dsl::*;
    
    let results = fonds
        .load::<models::Fond>(conn)?;
    
    Ok(results)
}

pub fn delete_fond(
    conn: &mut SqliteConnection,
    fond_number: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::fonds::dsl::*;
    
    diesel::delete(fonds.filter(fond_no.eq(fond_number)))
        .execute(conn)?;
    
    Ok(true)
}

// Series operations
pub fn create_series(
    conn: &mut SqliteConnection,
    ser: &models::Series,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::series::dsl::*;
    
    diesel::insert_into(series)
        .values(ser)
        .execute(conn)?;
    Ok(())
}

pub fn list_series_by_fond(
    conn: &mut SqliteConnection,
    fond_number: &str,
) -> Result<Vec<models::Series>, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::series::dsl::*;
    
    let results = series
        .filter(fond_no.eq(fond_number))
        .load::<models::Series>(conn)?;
    
    Ok(results)
}

// File operations
pub fn create_file(
    conn: &mut SqliteConnection,
    file: &models::File,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::files::dsl::*;
    
    diesel::insert_into(files)
        .values(file)
        .execute(conn)?;
    Ok(())
}

pub fn list_files_by_series(
    conn: &mut SqliteConnection,
    series_number: &str,
) -> Result<Vec<models::File>, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::files::dsl::*;
    
    let results = files
        .filter(series_no.eq(series_number))
        .load::<models::File>(conn)?;
    
    Ok(results)
}

pub fn delete_file(
    conn: &mut SqliteConnection,
    file_number: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::files::dsl::*;
    
    diesel::delete(files.filter(file_no.eq(file_number)))
        .execute(conn)?;
    
    Ok(true)
}
