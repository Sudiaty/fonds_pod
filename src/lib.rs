pub mod core;
pub mod models;
pub mod persistence;
pub mod services;
pub mod viewmodels;

// Re-export core traits for convenience
pub use core::crud_list_vm::ActiveableCrudViewModel;
pub use core::{
    Activeable, ActiveableRepository, Creatable, CrudViewModelBase, GenericRepository, Sortable,
    SortableRepository,
};

// Re-export viewmodels
pub use viewmodels::{AboutViewModel, ArchiveLibraryUIItem, SchemaViewModel, SettingsViewModel};

// Include Slint modules - this exports AppWindow, CrudListItem, DialogField, DialogFieldType, etc.
slint::include_modules!();
