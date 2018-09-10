extern crate pulldown_cmark as cmark;
extern crate pdf_canvas;

use pdf_canvas::{Pdf, BuiltinFont, FontSource};
use cmark::*;
use std::fs::File;
use std::io::Read;

/// PAGE_SIZE is the size of a sheet of A4 paper in pt
const PAGE_SIZE: (f32, f32) = (595.0, 842.0);
const MARGIN: (f32, f32) = (50.0, 50.0);
const DEFAULT_FONT: BuiltinFont = BuiltinFont::Times_Roman;
const BOLD_FONT: BuiltinFont = BuiltinFont::Times_Bold;
const ITALIC_FONT: BuiltinFont = BuiltinFont::Times_Italic;

const DEFAULT_FONT_SIZE: f32 = 12.0;
const H1_FONT_SIZE: f32 = 32.0;
const H2_FONT_SIZE: f32 = 28.0;
const H3_FONT_SIZE: f32 = 20.0;
const H4_FONT_SIZE: f32 = 16.0;

const LINE_SPACING: f32 = 1.75; // Text height * LINE_SPACING
const LIST_INDENTATION: f32 = 20.0;
const QUOTE_INDENTATION: f32 = 20.0;

const DEFAULT_OUTPUT_FILENAME: &str = "test.pdf";

#[derive(Clone, Debug)]
enum Span {
    Text {
        text: String,
        font_type: BuiltinFont,
        font_size: f32,
    },
}

impl Span {
    pub fn text(text: String, font_type: BuiltinFont, font_size: f32) -> Self {
        Span::Text {
            text, font_type, font_size
        }
    }

    pub fn width(&self) -> f32 {
        match self {
            Span::Text { text, font_type, font_size } => font_type.get_width(*font_size, text),
        }
    }

    pub fn height(&self) -> f32 {
        match self {
            Span::Text { font_size, .. } => *font_size,
        }
    }
}

#[derive(Clone, Debug)]
enum Section {
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
            Section::Plain(spans) => spans.iter().map(|x| x.height()).fold(0.0, |x, acc| acc.max(x)),
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

enum SubsectionType {
    List,
    Quote,
}

struct Lines {
    pub x: f32,
    lines: Vec<Section>,
    current_line: Vec<Span>,
    current_font: BuiltinFont,
    current_size: f32,
    max_width: f32,
    subsection: Option<Box<Lines>>,
    pub is_code: bool,
}

impl Lines {
    pub fn new(max_width: f32) -> Self {
        Self {
            x: 0.0,
            lines: Vec::new(),
            current_line: Vec::new(),
            current_font: DEFAULT_FONT,
            current_size: DEFAULT_FONT_SIZE,
            max_width: max_width,
            subsection: None,
            is_code: false,
        }
    }

    pub fn parse_event(&mut self, event: Event) -> Option<SubsectionType> {
        if self.subsection.is_some() {
            let mut subsection = self.subsection.take().unwrap();
            if let Some(sub_type) = subsection.parse_event(event) {
                let section = match sub_type {
                    SubsectionType::List => Section::list_item(subsection.get_vec()),
                    SubsectionType::Quote => Section::block_quote(subsection.get_vec()),
                };
                self.push_section(section);
            } else {
                self.subsection = Some(subsection);
            };
            return None;
        }
        match event {
            Event::Start(Tag::Strong) => self.current_font = BOLD_FONT,
            Event::End(Tag::Strong) => self.current_font = DEFAULT_FONT,
            Event::Start(Tag::Emphasis) => self.current_font = ITALIC_FONT,
            Event::End(Tag::Emphasis) => self.current_font = DEFAULT_FONT,

            Event::Start(Tag::Header(size)) => self.current_size = match size {
                1 => H1_FONT_SIZE,
                2 => H2_FONT_SIZE,
                3 => H3_FONT_SIZE,
                _ => H4_FONT_SIZE,
            },
            Event::End(Tag::Header(_)) => {
                self.current_size = DEFAULT_FONT_SIZE;
                self.new_line();
            },

            Event::Start(Tag::List(_)) => self.new_line(),
            Event::Start(Tag::Item) => self.subsection = Some(Box::new(Lines::new(self.max_width - LIST_INDENTATION))),
            Event::End(Tag::Item) => return Some(SubsectionType::List),

            Event::Start(Tag::BlockQuote) => {
                self.new_line();
                self.subsection = Some(Box::new(Lines::new(self.max_width - QUOTE_INDENTATION)))
            },
            Event::End(Tag::BlockQuote) => return Some(SubsectionType::Quote),

            Event::Text(ref text) if self.is_code => {
                let mut start = 0;
                for (pos, c) in text.chars().enumerate() {
                    if c == '\n' {
                        self.write(&text[start..pos]);
                        self.new_line();
                        start = pos + 1;
                    }
                }
                if start < text.len() {
                        self.write(&text[start..]);
                }
            },
            Event::Text(text) => self.write_left_aligned(&text),

            Event::Start(Tag::Code) => self.current_font = BuiltinFont::Courier,
            Event::End(Tag::Code) => self.current_font = DEFAULT_FONT,

            Event::Start(Tag::CodeBlock(_src_type)) => {
                self.is_code = true;
                self.current_font = BuiltinFont::Courier;
                self.current_size = DEFAULT_FONT_SIZE;
            },
            Event::End(Tag::CodeBlock(_)) => {
                self.push_section(Section::space(DEFAULT_FONT_SIZE));
                self.is_code = false;
                self.current_font = DEFAULT_FONT;
            },

            Event::Start(Tag::Paragraph) => {},
            Event::End(Tag::Paragraph) => {
                self.new_line();
                self.push_section(Section::space(DEFAULT_FONT_SIZE));
            },

            Event::SoftBreak => self.write(" "),
            Event::HardBreak => self.new_line(),

            _ => {}
        };
        None
    }

    pub fn push_section(&mut self, section: Section) {
        self.lines.push(section);
    }

    pub fn write_left_aligned(&mut self, text: &str) {
        let space_width = self.current_font.get_width(self.current_size, " ");

        let mut buffer = String::new();
        let mut buffer_width = 0.0;
        let mut pos = 0;
        while pos < text.len() {
            let idx = text[pos..].find(char::is_whitespace).unwrap_or(text.len()-pos-1)+pos+1;
            let word = &text[pos..idx];
            pos = idx;
            let word_width = self.current_font.get_width(self.current_size, word);
            if self.x + buffer_width + word_width > self.max_width {
                self.write(&buffer);
                self.new_line();
                buffer.clear();
                buffer_width = 0.0;
            }
            if buffer.len() > 0 {
                buffer.push(' ');
                buffer_width += space_width;
            }
            buffer.push_str(word);
            buffer_width += word_width;
        }
        let span = Span::text(buffer, self.current_font, self.current_size);
        self.push_span(span);
    }

    pub fn write(&mut self, text: &str) {
        let span = Span::text(text.into(), self.current_font, self.current_size);
        self.push_span(span);
    }

    pub fn push_span(&mut self, span: Span) {
        self.x += span.width();
        self.current_line.push(span);
    }

    pub fn new_line(&mut self) {
        if self.current_line.len() == 0 { return }
        self.lines.push(Section::plain(self.current_line.clone()));
        self.current_line.clear();
        self.x = 0.0;
    }

    pub fn get_vec(mut self) -> Vec<Section> {
        // Make sure that current_line is put into the output
        if self.current_line.len() != 0 {
            self.lines.push(Section::plain(self.current_line));
        }
        self.lines
    }
}

fn main() {
    let mut doc = Pdf::create(DEFAULT_OUTPUT_FILENAME).unwrap();

    let mut markdown_file = File::open("test.md").unwrap();
    let mut markdown = String::new();
    markdown_file.read_to_string(&mut markdown).unwrap();

    let parser = Parser::new(&markdown);

    let max_width = PAGE_SIZE.0 - MARGIN.0 - MARGIN.0;
    let mut lines = Lines::new(max_width);

    for event in parser {
        lines.parse_event(event);
    }

    let sections = lines.get_vec();

    for section in sections.iter() {
        match section {
            Section::Plain(v) => println!("plain: {:?}", v),
            Section::VerticalSpace(v) => println!("vertical_space: {:?}", v),
            Section::ListItem(v) => println!("list_item: {:?}", v),
            Section::BlockQuote(v) => println!("block_quote: {:?}", v),
        }
    }

    let mut pages = Pages::new();
    pages.render_sections(&sections[..], MARGIN.0);

    let pages = pages.into_vec();

    for page in pages {
        doc.render_page(PAGE_SIZE.0, PAGE_SIZE.1, |canvas| {
            let regular = canvas.get_font(DEFAULT_FONT);
            let bold = canvas.get_font(BOLD_FONT);
            let italic = canvas.get_font(ITALIC_FONT);
            let mono = canvas.get_font(BuiltinFont::Courier);
            canvas.text(|t| {
                let mut page = page.into_vec().into_iter().peekable();
                let mut pos = match page.peek() {
                    Some(x) => x.pos.clone(),
                    None => return Ok(()),
                };
                t.set_font(&regular, DEFAULT_FONT_SIZE)?;
                t.set_leading(18.0)?;
                t.pos(pos.0, pos.1)?;
                for span in page {
                    let delta = (span.pos.0-pos.0, span.pos.1-pos.1);
                    t.pos(delta.0, delta.1)?;
                    pos = span.pos;

                    match span.span {
                        Span::Text { text, font_type, font_size } => {
                            let font = match font_type {
                                BuiltinFont::Times_Roman => &regular,
                                BuiltinFont::Times_Bold => &bold,
                                BuiltinFont::Times_Italic => &italic,
                                BuiltinFont::Courier => &mono,
                                _ => &regular,
                            };
                            t.set_font(font, font_size)?;
                            t.show(&text)?;
                        }
                    }
                }
                Ok(())
            })
        }).unwrap();
    }

    doc.finish().unwrap();
}

#[derive(Clone)]
struct Page {
    positioned_spans: Vec<PositionedSpan>,
}

impl Page {
    pub fn new() -> Self {
        Self {
            positioned_spans: vec![],
        }
    }

    pub fn render_spans(&mut self, spans: &[Span], start_x: f32, start_y: f32) {
        let mut x = start_x;
        let y = start_y;
        for span in spans {
            self.positioned_spans.push(PositionedSpan::new(span.clone(), x, y));
            x += span.width();
        }
    }

    pub fn clear(&mut self) {
        self.positioned_spans.clear();
    }

    pub fn into_vec(self) -> Vec<PositionedSpan> {
        self.positioned_spans
    }
}

struct Pages {
    pages: Vec<Page>,
    current_page: Page,
    current_y: f32,
}

impl Pages {
    pub fn new() -> Self {
        Self {
            pages: vec![],
            current_page: Page::new(),
            current_y: PAGE_SIZE.1 - MARGIN.1,
        }
    }

    fn new_page(&mut self) {
        self.pages.push(self.current_page.clone());
        self.current_page.clear();
        self.current_y = PAGE_SIZE.1 - MARGIN.1;
    }

    pub fn render_sections(&mut self, sections: &[Section], start_x: f32) {
        let min_y = MARGIN.1;
        for section in sections {
            let height = section.min_step();
            let delta_y = -height * LINE_SPACING;
            if self.current_y + delta_y < min_y {
                self.new_page();
            }
            self.current_y += delta_y;
            match section {
                Section::Plain(spans) => self.current_page.render_spans(&spans, start_x, self.current_y),
                Section::VerticalSpace(_) => {}
                Section::ListItem(ref sections) => {
                    self.current_page.render_spans(&[Span::text("o".into(), DEFAULT_FONT, DEFAULT_FONT_SIZE)], start_x, self.current_y);
                    self.current_y -= delta_y;
                    self.render_sections(sections, start_x + LIST_INDENTATION);
                },
                Section::BlockQuote(ref sections) => {
                    self.current_page.render_spans(&[Span::text("|".into(), DEFAULT_FONT, DEFAULT_FONT_SIZE)], start_x, self.current_y);
                    self.current_y -= delta_y;
                    self.render_sections(sections, start_x + QUOTE_INDENTATION);
                },
            }
        }
    }

    pub fn into_vec(mut self) -> Vec<Page> {
        self.pages.push(self.current_page);
        self.pages
    }
}

#[derive(Clone)]
struct PositionedSpan {
    pub span: Span,
    pub pos: (f32, f32),
}

impl PositionedSpan {
    pub fn new(span: Span, x: f32, y: f32) -> Self {
        let pos = (x, y);
        Self { span, pos }
    }
}
