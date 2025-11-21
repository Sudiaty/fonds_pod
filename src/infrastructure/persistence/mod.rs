/// Persistence layer - Database operations (queries, schema)
/// This module encapsulates all database-related code
pub mod models;
pub mod queries;
pub mod schema;

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
