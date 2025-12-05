pub mod schema;
pub mod schema_repository;
pub mod schema_item_repository;
pub mod fond_classification_repository;

// Re-export core traits for convenience
pub use crate::core::generic_repository::{Creatable, GenericRepository};
pub use fond_classification_repository::FondClassificationsRepository;

use diesel::sqlite::SqliteConnection;
use diesel::Connection;
use std::error::Error;
use std::path::Path;

/// Initialize database connection
pub fn establish_connection(database_path: &Path) -> Result<SqliteConnection, Box<dyn Error>> {
    let database_url = database_path.to_string_lossy().to_string();
    let mut connection = SqliteConnection::establish(&database_url)?;
    schema::init_schema(&mut connection)?;
    Ok(connection)
}