/// Files service - coordinates file operations with business rules
use crate::infrastructure::persistence::{queries, establish_connection};
use crate::infrastructure::persistence::models::{File, Item};
use std::error::Error;
use std::path::Path;
use chrono::Local;

/// Files service with direct database access
pub struct FileService;

/// Data structure for creating a new file
pub struct CreateFileInput {
    /// The series number this file belongs to
    pub series_no: String,
    /// Name of the file
    pub name: String,
    /// Creation date (defaults to current date if None)
    pub created_at: Option<String>,
}

/// Result of creating a file
pub struct CreateFileResult {
    pub file_no: String,
}

/// Data structure for creating a new item
pub struct CreateItemInput {
    /// The file number this item belongs to
    pub file_no: String,
    /// Name of the item (usually the file/folder name)
    pub name: String,
    /// Path to the file or folder
    pub path: Option<String>,
    /// Creation date (defaults to current date if None)
    pub created_at: Option<String>,
}

/// Result of creating an item
pub struct CreateItemResult {
    pub item_no: String,
}

impl FileService {
    /// Create a new file
    pub fn create_file(
        db_path: &Path,
        input: CreateFileInput,
    ) -> Result<CreateFileResult, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;

        // Generate file number
        let seq = queries::get_next_sequence(&mut conn, &input.series_no)?;
        let formatted_seq = queries::format_sequence(seq, 2);
        let file_no = format!("{}-{}", input.series_no, formatted_seq);

        // Determine created_at
        let created_at = input.created_at.unwrap_or_else(|| {
            Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
        });

        // Create file record
        let file = File {
            file_no: file_no.clone(),
            series_no: input.series_no,
            name: input.name,
            created_at,
        };
        queries::create_file(&mut conn, &file)?;

        Ok(CreateFileResult { file_no })
    }

    /// Delete a file
    pub fn delete_file(db_path: &Path, file_no: &str) -> Result<bool, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        queries::delete_file(&mut conn, file_no)
    }

    /// Create a new item
    pub fn create_item(
        db_path: &Path,
        input: CreateItemInput,
    ) -> Result<CreateItemResult, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;

        // Generate item number
        let seq = queries::get_next_sequence(&mut conn, &input.file_no)?;
        let formatted_seq = queries::format_sequence(seq, 2);
        let item_no = format!("{}-{}", input.file_no, formatted_seq);

        // Determine created_at
        let created_at = input.created_at.unwrap_or_else(|| {
            Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
        });

        // Create item record
        let item = Item {
            item_no: item_no.clone(),
            file_no: input.file_no,
            name: input.name,
            path: input.path,
            created_at,
        };
        queries::create_item(&mut conn, &item)?;

        Ok(CreateItemResult { item_no })
    }

    /// List all items for a file
    pub fn list_items_by_file(db_path: &Path, file_no: &str) -> Result<Vec<Item>, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        queries::list_items_by_file(&mut conn, file_no)
    }

    /// Delete an item
    pub fn delete_item(db_path: &Path, item_no: &str) -> Result<bool, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        queries::delete_item(&mut conn, item_no)
    }
}