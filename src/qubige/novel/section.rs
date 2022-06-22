pub struct Section {
    pub name: String,
    short_link: String,
}

impl Section {
    pub fn new(name: String, short_link: String) -> Self {
        Self{
            name,
            short_link
        }
    }
}