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

pub fn list_fonds(
    conn: &mut SqliteConnection,
) -> Result<Vec<models::Fond>, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::fonds::dsl::*;
    
    let results = fonds
        .load::<models::Fond>(conn)?;
    
    Ok(results)
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

pub fn rename_file(
    conn: &mut SqliteConnection,
    file_number: &str,
    new_name: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::files::dsl::*;
    
    diesel::update(files.filter(file_no.eq(file_number)))
        .set(name.eq(new_name))
        .execute(conn)?;
    
    Ok(true)
}

pub fn list_items_by_file(
    conn: &mut SqliteConnection,
    file_number: &str,
) -> Result<Vec<models::Item>, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::items::dsl::*;
    
    // Try to load with path column first (new schema)
    let results = items
        .filter(file_no.eq(file_number))
        .load::<models::Item>(conn);
    
    // If it fails due to missing path column, try without it
    match results {
        Ok(loaded_items) => Ok(loaded_items),
        Err(_) => {
            // Fall back to raw SQL that handles missing path column
            diesel::sql_query(
                "SELECT item_no, file_no, name, NULL as path, created_at FROM items WHERE file_no = ?"
            )
            .bind::<diesel::sql_types::Text, _>(file_number)
            .load::<models::Item>(conn)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
    }
}

pub fn create_item(
    conn: &mut SqliteConnection,
    item: &models::Item,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::items::dsl::*;
    
    // Try to insert with path column
    let insert_result = diesel::insert_into(items)
        .values(item)
        .execute(conn);
    
    // If it fails due to missing path column, insert without it
    match insert_result {
        Ok(_) => Ok(()),
        Err(_) => {
            // Fall back to inserting without path column
            diesel::sql_query(
                "INSERT INTO items (item_no, file_no, name, created_at) VALUES (?, ?, ?, ?)"
            )
            .bind::<diesel::sql_types::Text, _>(&item.item_no)
            .bind::<diesel::sql_types::Text, _>(&item.file_no)
            .bind::<diesel::sql_types::Text, _>(&item.name)
            .bind::<diesel::sql_types::Text, _>(&item.created_at)
            .execute(conn)?;
            Ok(())
        }
    }
}

pub fn delete_item(
    conn: &mut SqliteConnection,
    item_number: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::items::dsl::*;
    
    diesel::delete(items.filter(item_no.eq(item_number)))
        .execute(conn)?;
    
    Ok(true)
}

pub fn rename_item(
    conn: &mut SqliteConnection,
    item_number: &str,
    new_name: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::items::dsl::*;
    
    diesel::update(items.filter(item_no.eq(item_number)))
        .set(name.eq(new_name))
        .execute(conn)?;
    
    Ok(true)
}

// ============================================================================
// Sequence operations for generating unique numbers
// ============================================================================

/// Get the next sequence value for a given prefix
/// If the prefix doesn't exist, it will be created with initial value 1
pub fn get_next_sequence(
    conn: &mut SqliteConnection,
    prefix_value: &str,
) -> Result<i32, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::sequences::dsl::*;
    
    // Try to get existing sequence
    let result = sequences
        .filter(prefix.eq(prefix_value))
        .first::<models::Sequence>(conn)
        .optional()?;
    
    let next_value = match result {
        Some(seq) => seq.current_value + 1,
        None => 1,
    };
    
    // Upsert the sequence
    diesel::replace_into(sequences)
        .values(&models::Sequence {
            prefix: prefix_value.to_string(),
            current_value: next_value,
        })
        .execute(conn)?;
    
    Ok(next_value)
}

/// Format sequence value with specified digits (default 2)
pub fn format_sequence(value: i32, digits: usize) -> String {
    format!("{:0>width$}", value, width = digits)
}

// ============================================================================
// FondSchema operations
// ============================================================================

/// Create a fond-schema association
pub fn create_fond_schema(
    conn: &mut SqliteConnection,
    fond_schema: &models::FondSchema,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::fond_schemas::dsl::*;
    
    diesel::insert_into(fond_schemas)
        .values(fond_schema)
        .execute(conn)?;
    Ok(())
}

/// List all schemas associated with a fond, ordered by order_no
pub fn list_fond_schemas(
    conn: &mut SqliteConnection,
    fond_number: &str,
) -> Result<Vec<models::FondSchema>, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::fond_schemas::dsl::*;
    
    let results = fond_schemas
        .filter(fond_no.eq(fond_number))
        .order(order_no.asc())
        .load::<models::FondSchema>(conn)?;
    
    Ok(results)
}

/// Check if a series already exists
pub fn series_exists(
    conn: &mut SqliteConnection,
    series_number: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    use crate::infrastructure::persistence::schema::series::dsl::*;
    
    let count = series
        .filter(series_no.eq(series_number))
        .count()
        .get_result::<i64>(conn)?;
    
    Ok(count > 0)
}
