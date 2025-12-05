pub mod generic_repository;
pub mod activeable_repository;

pub use generic_repository::{Creatable, GenericRepository};
pub use activeable_repository::{Activeable, ActiveableRepository};