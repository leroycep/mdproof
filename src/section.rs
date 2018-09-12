use span::Span;

#[derive(Clone, Debug)]
pub enum Section {
    Plain(Vec<Span>),
    VerticalSpace(f32),
    ListItem(Vec<Section>),
    BlockQuote(Vec<Section>),
}

impl Section {
    pub fn plain(spans: Vec<Span>) -> Self {
        Section::Plain(spans)
    }

    pub fn space(space_pt: f32) -> Self {
        Section::VerticalSpace(space_pt)
    }

    pub fn list_item(sections: Vec<Section>) -> Self {
        Section::ListItem(sections)
    }

    pub fn block_quote(sections: Vec<Section>) -> Self {
        Section::BlockQuote(sections)
    }

    pub fn height(&self) -> f32 {
        match self {
            Section::Plain(spans) => spans
                .iter()
                .map(|x| x.height())
                .fold(0.0, |x, acc| acc.max(x)),
            Section::VerticalSpace(space_pt) => *space_pt,
            Section::ListItem(sections) => sections.iter().map(|x| x.height()).sum(),
            Section::BlockQuote(sections) => sections.iter().map(|x| x.height()).sum(),
        }
    }

    pub fn min_step(&self) -> f32 {
        match self {
            Section::Plain(_) => self.height(),
            Section::VerticalSpace(_) => self.height(),
            Section::ListItem(sections) => sections.iter().take(1).map(|x| x.height()).sum(),
            Section::BlockQuote(sections) => sections.iter().take(1).map(|x| x.height()).sum(),
        }
    }
}
