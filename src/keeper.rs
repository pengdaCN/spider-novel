use crate::spider::Spider;

#[derive(Default)]
pub struct Keeper {
    spiders: Vec<Box<dyn Spider>>,
}

impl Keeper {
    pub fn new() -> Self {
        Keeper::default()
    }

    pub fn add_spider(&mut self, spider: Box<dyn Spider>) {
        self.spiders.push(spider)
    }
}