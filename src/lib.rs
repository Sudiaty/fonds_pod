pub mod core;
pub mod models;
pub mod persistence;
pub mod viewmodels;
pub mod services;

// Re-export core traits for convenience
pub use core::{
    Creatable, GenericRepository, Activeable, ActiveableRepository, Sortable, SortableRepository,
    CrudViewModelBase,
};

// Re-export viewmodels
pub use viewmodels::{SchemaViewModel, SettingsViewModel, ArchiveLibraryUIItem, AboutViewModel};

// Include Slint modules - this exports AppWindow, CrudListItem, DialogField, DialogFieldType, etc.
slint::include_modules!();