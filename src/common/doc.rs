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

impl <'a> WrapSelection<'a> {
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

    pub fn iter<'x >(&'x self) -> impl Iterator<Item=WrapSelection<'a>> + 'x {
        self.inner.iter().map(|x| Self{
            inner: x
        })
    }
}
