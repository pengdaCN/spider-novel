pub mod sort;

use crate::spider::Sort;

pub trait SortRepo {
    fn add(&mut self, id: &str, sorts: Vec<&Sort>);
}