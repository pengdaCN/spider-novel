use chrono::format::Item;
use nipper::{Document, Node, Selection, Selections};
use skyscraper::html;
use skyscraper::html::parse::ParseError;
use skyscraper::html::HtmlDocument;

pub fn parse(doc: &str) -> Result<HtmlDocument, ParseError> {
    let x = Document::from(doc);
    html::parse(&x.html().to_string())
}

pub struct WrapDocument {
    inner: Document,
}

unsafe impl Sync for WrapDocument {}

unsafe impl Send for WrapDocument {}

pub struct WrapSelection<'a> {
    inner: Selection<'a>,
}

unsafe impl Sync for WrapSelection<'_> {}

unsafe impl Send for WrapSelection<'_> {}

pub struct WrapSections<'a> {
    inner: Selections<Node<'a>>,
}

unsafe impl Sync for WrapSections<'_> {}

unsafe impl Send for WrapSections<'_> {}

impl WrapDocument {
    pub fn parse(doc: &str) -> Self {
        Self {
            inner: Document::from(doc),
        }
    }

    pub fn select(&self, sel: &str) -> WrapSelection<'_> {
        WrapSelection {
            inner: self.inner.select(sel),
        }
    }
}

impl<'a> WrapSelection<'a> {
    pub fn text(&self) -> String {
        self.inner.text().to_string()
    }

    pub fn attr(&self, name: &str) -> Option<String> {
        self.inner.attr(name).map(|x| x.to_string())
    }

    pub fn select(&self, sel: &str) -> Self {
        Self {
            inner: self.inner.select(sel),
        }
    }

    pub fn iter(&self) -> WrapSections<'a> {
        WrapSections {
            inner: self.inner.iter(),
        }
    }

    pub fn children(&self) -> Self {
        Self {
            inner: self.inner.children(),
        }
    }

    pub fn parent(&self) -> Self {
        Self {
            inner: self.inner.parent(),
        }
    }
}

impl<'a> Iterator for WrapSections<'a> {
    type Item = WrapSelection<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| Self::Item { inner: x })
    }
}
