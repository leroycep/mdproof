use printpdf::Mm;
use resources::Resources;
use span::Span;

#[derive(Clone, Debug)]
pub enum Section {
    Plain(Vec<Span>),
    VerticalSpace(Mm),
    ThematicBreak,
    PageBreak,
    ListItem(Vec<Section>),
    BlockQuote(Vec<Section>),
    CodeBlock(Vec<Vec<Span>>),
}

impl Section {
    pub fn plain(spans: Vec<Span>) -> Self {
        Section::Plain(spans)
    }

    pub fn space(height: Mm) -> Self {
        Section::VerticalSpace(height)
    }

    pub fn list_item(sections: Vec<Section>) -> Self {
        Section::ListItem(sections)
    }

    pub fn block_quote(sections: Vec<Section>) -> Self {
        Section::BlockQuote(sections)
    }

    pub fn code_block(lines: Vec<Vec<Span>>) -> Self {
        Section::CodeBlock(lines)
    }

    pub fn page_break() -> Self {
        Section::PageBreak
    }

    pub fn height(&self, resources: &Resources) -> Mm {
        let r = match self {
            Section::Plain(spans) => spans
                .iter()
                .map(|x| x.height(resources).0)
                .fold(0.0, |x, acc| acc.max(x)),
            Section::VerticalSpace(space_pt) => space_pt.0,
            Section::ThematicBreak => 0.0,
            Section::PageBreak => 0.0,
            Section::ListItem(sections) => sections.iter().map(|x| x.height(resources).0).sum(),
            Section::BlockQuote(sections) => sections.iter().map(|x| x.height(resources).0).sum(),
            Section::CodeBlock(lines) => lines
                .iter()
                .map(|line| {
                    line.iter()
                        .map(|x| x.height(resources).0)
                        .fold(0.0, |x, acc| acc.max(x))
                })
                .sum(),
        };
        Mm(r)
    }

    pub fn min_step(&self, resources: &Resources) -> Mm {
        let r = match self {
            Section::Plain(_) => self.height(resources).0,
            Section::VerticalSpace(_) => self.height(resources).0,
            Section::ThematicBreak => self.height(resources).0,
            Section::PageBreak => self.height(resources).0,
            Section::ListItem(sections) => {
                sections.iter().take(1).map(|x| x.height(resources).0).sum()
            }
            Section::BlockQuote(sections) => {
                sections.iter().take(1).map(|x| x.height(resources).0).sum()
            }
            Section::CodeBlock(lines) => lines
                .iter()
                .take(1)
                .flat_map(|x| x.iter())
                .map(|x| x.height(resources).0)
                .sum(),
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
            Section::CodeBlock(_lines) => false,
        }
    }
}
