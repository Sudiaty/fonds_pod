use slint::{Weak, ComponentHandle};

pub struct SchemaViewModel {
}

impl SchemaViewModel {
    pub fn new<T: ComponentHandle>(_ui_handle: Weak<T>) -> Self {
        SchemaViewModel {}
    }
}