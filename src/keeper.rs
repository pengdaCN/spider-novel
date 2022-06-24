use crate::spider::{Spider, SpiderMetadata, Support};

#[derive(Debug, Default)]
pub struct Policy {}

struct PropertySpider {
    id: &'static str,
    supported: &'static Support,
    inner: Box<dyn Spider>,
}

impl PropertySpider {
    fn new(id: &'static str, supported: &'static Support, inner: Box<dyn Spider>) -> Self {
        Self {
            id,
            supported,
            inner,
        }
    }
}

#[derive(Default)]
pub struct Keeper {
    spiders: Vec<PropertySpider>,
    policy: Policy,
}

impl Keeper {
    pub fn new() -> Self {
        Keeper::default()
    }

    pub fn add_spider<T>(&mut self, spider: T)
        where T: SpiderMetadata + Spider + 'static
    {
        self.spiders.push(PropertySpider::new(T::id(), &T::SUPPORTED, Box::new(spider)))
    }
}