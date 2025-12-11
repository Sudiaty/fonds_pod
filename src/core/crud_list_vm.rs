use crate::{Creatable, core::GenericRepository, CrudListItem};
use slint::{ModelRc, Model, VecModel};
use std::rc::Rc;
use std::cell::RefCell;
use std::marker::PhantomData;

/// Trait to convert an entity to a CrudListItem
pub trait ToCrudListItem {
    fn to_crud_list_item(&self) -> CrudListItem;
}

pub struct CrudViewModel<T, R>
where
    T: Creatable + Clone + Default + ToCrudListItem + 'static,
    R: GenericRepository<T> + 'static,
{
    pub items: Rc<VecModel<CrudListItem>>,
    repo: Rc<RefCell<R>>,
    _phantom: PhantomData<T>,
}

impl<T, R> Clone for CrudViewModel<T, R>
where
    T: Creatable + Clone + Default + ToCrudListItem + 'static,
    R: GenericRepository<T> + 'static,
{
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            repo: self.repo.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T, R> CrudViewModel<T, R>
where
    T: Creatable + Clone + Default + ToCrudListItem + 'static,
    R: GenericRepository<T> + 'static,
{
    pub fn new(repo: Rc<RefCell<R>>) -> Self {
        Self {
            items: Rc::new(VecModel::default()),
            repo,
            _phantom: PhantomData,
        }
    }

    pub fn load(&self) {
        let items = self.repo.borrow_mut().find_all().unwrap_or_default();
        let crud_items: Vec<CrudListItem> = items.iter().map(|item| item.to_crud_list_item()).collect();
        self.items.set_vec(crud_items);
    }

    pub fn add(&self, mut item: T) {
        let mut repo = self.repo.borrow_mut();
        if let Ok(id) = repo.create(item.clone()) {
            item.set_id(id);
            self.items.push(item.to_crud_list_item());
        }
    }

    pub fn delete(&self, index: usize) {
        if let Some(item) = self.items.row_data(index) {
             let mut repo = self.repo.borrow_mut();
             if repo.delete(item.id).is_ok() {
                 self.items.remove(index);
             }
        }
    }
    
    pub fn get_items(&self) -> ModelRc<CrudListItem> {
        ModelRc::from(self.items.clone())
    }
}