extern crate pulldown_cmark as cmark;
extern crate pdf_canvas;

use pdf_canvas::{Pdf, BuiltinFont, FontSource};
use cmark::*;
use std::fs::File;
use std::io::Read;
use std::collections::VecDeque;
use std::borrow::Cow;

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
enum Span<'txt> {
    Text {
        text: Cow<'txt, str>,
        font_type: BuiltinFont,
        font_size: f32,
    },
}

impl<'txt> Span<'txt> {
    pub fn text(text: Cow<'txt, str>, font_type: BuiltinFont, font_size: f32) -> Self {
        Span::Text {
            text, font_type, font_size
        }
    }
}

struct Lines<'txt> {
    pub x: f32,
    lines: VecDeque<Vec<Span<'txt>>>,
    current_line: Vec<Span<'txt>>,
}

impl<'txt> Lines<'txt> {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            lines: VecDeque::new(),
            current_line: Vec::new(),
        }
    }

    pub fn push_span(&mut self, span: Span<'txt>, width: f32) {
        self.x += width;
        self.current_line.push(span);
    }

    pub fn new_line(&mut self) {
        self.lines.push_back(self.current_line.clone());
        self.current_line.clear();
        self.x = 0.0;
    }

    pub fn get_vecdeque(self) -> VecDeque<Vec<Span<'txt>>> {
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

            Event::Text(text) => {
                let width = current_font.get_width(current_size, &text);
                if lines.x + width > max_width {
                    lines.new_line();
                }
                lines.push_span(Span::text(text, current_font, current_size), width);
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
            canvas.text(|t| {
                t.set_font(&regular, DEFAULT_FONT_SIZE)?;
                t.set_leading(18.0)?;
                t.pos(MARGIN.0, PAGE_SIZE.1-MARGIN.1)?;
                let mut y = PAGE_SIZE.1-MARGIN.1;
                let min_y = MARGIN.1;

                while y > min_y {
                    let line = match lines.pop_front() {
                        Some(l) => l,
                        None => break,
                    };
                    for span in line {
                        match span {
                            Span::Text { text, font_type, font_size } => {
                                let font = match font_type {
                                    BuiltinFont::Times_Roman => &regular,
                                    BuiltinFont::Times_Bold => &bold,
                                    BuiltinFont::Times_Italic => &italic,
                                    _ => &regular,
                                };
                                t.set_font(font, font_size)?;
                                t.show(&text)?;
                            }
                        }
                    }
                    t.show_line("")?;
                    y -= 18.0;
                }
                Ok(())
            })
        }).unwrap();
    }

    doc.finish().unwrap();
}
