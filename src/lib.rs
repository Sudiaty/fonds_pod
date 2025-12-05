pub mod core;
pub mod models;
pub mod persistence;
pub mod viewmodels;

// Re-export core traits for convenience
pub use core::{Creatable, GenericRepository, Activeable, ActiveableRepository};

slint::include_modules!();