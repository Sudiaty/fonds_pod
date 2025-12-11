pub mod schema;
pub mod schema_repository;
pub mod schema_item_repository;
pub mod fond_classification_repository;
pub mod fond_repository;
pub mod fond_schema_repository;
pub mod series_repository;
pub mod file_repository;
pub mod item_repository;
pub mod config_repository;

// Re-export core traits for convenience
pub use crate::core::generic_repository::{Creatable, GenericRepository};
pub use crate::core::sortable_repository::{Sortable, SortableRepository};
pub use crate::core::activeable_repository::{Activeable, ActiveableRepository};
pub use fond_classification_repository::FondClassificationsRepository;
pub use fond_repository::FondsRepository;
pub use fond_schema_repository::FondSchemasRepository;
pub use series_repository::SeriesRepository;
pub use file_repository::FilesRepository;
pub use item_repository::ItemsRepository;
pub use config_repository::FileConfigRepository;

use diesel::sqlite::SqliteConnection;
use diesel::Connection;
use std::error::Error;
use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;

/// Initialize database connection
pub fn establish_connection(database_path: &Path) -> Result<Rc<RefCell<SqliteConnection>>, Box<dyn Error>> {
    let database_url = database_path.to_string_lossy().to_string();
    let mut connection = SqliteConnection::establish(&database_url)?;
    schema::init_schema(&mut connection)?;
    Ok(Rc::new(RefCell::new(connection)))
}