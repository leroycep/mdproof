use printpdf::Mm;
use span::Span;

#[derive(Clone, Debug)]
pub enum Section<'collection> {
    Plain(Vec<Span<'collection>>),
    VerticalSpace(Mm),
    ThematicBreak,
    PageBreak,
    ListItem(Vec<Section<'collection>>),
    BlockQuote(Vec<Section<'collection>>),
}

impl<'collection> Section<'collection> {
    pub fn plain(spans: Vec<Span<'collection>>) -> Self {
        Section::Plain(spans)
    }

    pub fn space(height: Mm) -> Self {
        Section::VerticalSpace(height)
    }

    pub fn list_item(sections: Vec<Section<'collection>>) -> Self {
        Section::ListItem(sections)
    }

    pub fn block_quote(sections: Vec<Section<'collection>>) -> Self {
        Section::BlockQuote(sections)
    }

    pub fn page_break() -> Self {
        Section::PageBreak
    }

    pub fn height(&self) -> Mm {
        let r = match self {
            Section::Plain(spans) => spans
                .iter()
                .map(|x| x.height().0)
                .fold(0.0, |x, acc| acc.max(x)),
            Section::VerticalSpace(space_pt) => space_pt.0,
            Section::ThematicBreak => 0.0,
            Section::PageBreak => 0.0,
            Section::ListItem(sections) => sections.iter().map(|x| x.height().0).sum(),
            Section::BlockQuote(sections) => sections.iter().map(|x| x.height().0).sum(),
        };
        Mm(r)
    }

    pub fn min_step(&self) -> Mm {
        let r = match self {
            Section::Plain(_) => self.height().0,
            Section::VerticalSpace(_) => self.height().0,
            Section::ThematicBreak => self.height().0,
            Section::PageBreak => self.height().0,
            Section::ListItem(sections) => sections.iter().take(1).map(|x| x.height().0).sum(),
            Section::BlockQuote(sections) => sections.iter().take(1).map(|x| x.height().0).sum(),
        };
        Mm(r)
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Section::PageBreak => true,
            Section::VerticalSpace(_) => true,
            Section::ThematicBreak => false,
            Section::Plain(spans) => spans.len() == 0,
            Section::ListItem(_sections) => false,
            Section::BlockQuote(_sections) => false,
        }
    }
}
