extern crate pulldown_cmark as cmark;
extern crate pdf_canvas;

use pdf_canvas::{Pdf, BuiltinFont, FontSource};
use cmark::*;
use std::fs::File;
use std::io::Read;
use std::collections::VecDeque;

/// PAGE_SIZE is the size of a sheet of A4 paper in pt
const PAGE_SIZE: (f32, f32) = (595.0, 842.0);
const MARGIN: (f32, f32) = (20.0, 20.0);
const DEFAULT_FONT: BuiltinFont = BuiltinFont::Times_Roman;
const BOLD_FONT: BuiltinFont = BuiltinFont::Times_Bold;
const ITALIC_FONT: BuiltinFont = BuiltinFont::Times_Italic;

const DEFAULT_FONT_SIZE: f32 = 12.0;
const H1_FONT_SIZE: f32 = 28.0;
const H2_FONT_SIZE: f32 = 24.0;
const H3_FONT_SIZE: f32 = 20.0;
const H4_FONT_SIZE: f32 = 16.0;

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

    pub fn height(&self) -> f32 {
        match self {
            Span::Text { font_size, .. } => *font_size,
        }
    }
}

enum Section {
    Plain(Vec<Span>),
}

impl Section {
    pub fn plain(spans: Vec<Span>) -> Self {
        Section::Plain(spans)
    }

    pub fn height(&self) -> f32 {
        match self {
            Section::Plain(spans) => spans.iter().map(|x| x.height()).fold(0.0, |x, acc| acc.max(x)),
        }
    }
}

struct Lines {
    pub x: f32,
    lines: VecDeque<Section>,
    current_line: Vec<Span>,
    pub is_code: bool,
}

impl Lines {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            lines: VecDeque::new(),
            current_line: Vec::new(),
            is_code: false,
        }
    }

    pub fn push_span(&mut self, span: Span, width: f32) {
        self.x += width;
        self.current_line.push(span);
    }

    pub fn new_line(&mut self) {
        self.lines.push_back(Section::plain(self.current_line.clone()));
        self.current_line.clear();
        self.x = 0.0;
    }

    pub fn get_vecdeque(self) -> VecDeque<Section> {
        self.lines
    }
}

fn main() {
    let mut doc = Pdf::create(DEFAULT_OUTPUT_FILENAME).unwrap();

    let mut markdown_file = File::open("test.md").unwrap();
    let mut markdown = String::new();
    markdown_file.read_to_string(&mut markdown).unwrap();

    let parser = Parser::new(&markdown);

    let mut lines = Lines::new();
    let max_width = PAGE_SIZE.0 - MARGIN.0 - MARGIN.0;
    let mut current_font = DEFAULT_FONT;
    let mut current_size = DEFAULT_FONT_SIZE;

    for event in parser {
        match event {
            Event::Start(Tag::Strong) => current_font = BOLD_FONT,
            Event::End(Tag::Strong) => current_font = DEFAULT_FONT,
            Event::Start(Tag::Emphasis) => current_font = ITALIC_FONT,
            Event::End(Tag::Emphasis) => current_font = DEFAULT_FONT,

            Event::Start(Tag::Header(size)) => current_size = match size {
                1 => H1_FONT_SIZE,
                2 => H2_FONT_SIZE,
                3 => H3_FONT_SIZE,
                _ => H4_FONT_SIZE,
            },
            Event::End(Tag::Header(_)) => {
                current_size = DEFAULT_FONT_SIZE;
                lines.new_line();
            },

            Event::Start(Tag::Item) => lines.push_span(Span::text(" - ".into(), current_font, current_size), current_font.get_width(current_size, " - ")),
            Event::End(Tag::Item) => lines.new_line(),

            Event::Text(ref text) if lines.is_code => {
                let mut start = 0;
                for (pos, c) in text.chars().enumerate() {
                    if c == '\n' {
                        // We can put 0.0 for the width because we the call to new_line will make it irrelevant
                        lines.push_span(Span::text(text[start..pos].into(), current_font, current_size), 0.0);
                        lines.new_line();
                        start = pos + 1;
                    }
                }
                if start < text.len() {
                        lines.push_span(Span::text(text[start..].into(), current_font, current_size), 0.0);
                }
            },
            Event::Text(text) => {
                let width = current_font.get_width(current_size, &text);
                if lines.x + width > max_width {
                    lines.new_line();
                }
                lines.push_span(Span::text(text.to_string(), current_font, current_size), width);
            },

            Event::Start(Tag::CodeBlock(_src_type)) => {
                lines.is_code = true;
                current_font = BuiltinFont::Courier;
                current_size = DEFAULT_FONT_SIZE;
            },
            Event::End(Tag::CodeBlock(_)) => {
                lines.is_code = false;
                current_font = DEFAULT_FONT;
            },

            Event::Start(Tag::Paragraph) => lines.new_line(),
            Event::End(Tag::Paragraph) => lines.new_line(),

            Event::SoftBreak => lines.push_span(Span::text(" ".into(), current_font, current_size), current_font.get_width(current_size, " ")),
            Event::HardBreak => lines.new_line(),

            _ => {}
        }
    }

    let mut lines = lines.get_vecdeque();

    while lines.len() > 0 {
        doc.render_page(PAGE_SIZE.0, PAGE_SIZE.1, |canvas| {
            let regular = canvas.get_font(DEFAULT_FONT);
            let bold = canvas.get_font(BOLD_FONT);
            let italic = canvas.get_font(ITALIC_FONT);
            let mono = canvas.get_font(BuiltinFont::Courier);
            canvas.text(|t| {
                t.set_font(&regular, DEFAULT_FONT_SIZE)?;
                t.set_leading(18.0)?;
                t.pos(MARGIN.0, PAGE_SIZE.1-MARGIN.1)?;
                let mut y = PAGE_SIZE.1-MARGIN.1;
                let min_y = MARGIN.1;
                let spacing = 1.75;

                while y > min_y {
                    let line = match lines.pop_front() {
                        Some(l) => l,
                        None => break,
                    };
                    let height = line.height();
                    let delta_y = -height * spacing;
                    if y + delta_y < min_y {
                        lines.push_front(line);
                        break;
                    }
                    y += delta_y;
                    t.pos(0.0, delta_y)?;
                    match line {
                        Section::Plain(spans) => {
                            for span in spans {
                                match span {
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
                        }
                    }
                }
                Ok(())
            })
        }).unwrap();
    }

    doc.finish().unwrap();
}
