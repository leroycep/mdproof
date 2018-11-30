
use std::collections::HashSet;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Style(HashSet<Class>);

#[derive(Debug, Hash, Clone, Eq, PartialEq)]
pub enum Class {
    Heading(u8),
    /// BlockQuote with level of quotation
    BlockQuote(u8),
    Strong,
    Emphasis,
    Code,
    Note,
    Link,
    Superscript,
}

impl Style {
    pub fn insert(&mut self, class: Class) {
        self.0.insert(class);
    }

    pub fn remove(&mut self, class: &Class) {
        self.0.remove(class);
    }
}

impl Default for Style {
    fn default() -> Self {
        Style(HashSet::new())
    }
}

impl<'a, I: Iterator<Item=&'a Class>> From<I> for Style {
    fn from(classes: I) -> Self {
        let mut style = Style::default();
        for c in classes {
            style.insert(c.clone());
        }
        style
    }
}

impl ::std::hash::Hash for Style {
    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
        for class in self.0.iter() {
            class.hash(state);
        }
    }
}

