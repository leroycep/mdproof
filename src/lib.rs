extern crate pdf_canvas;
extern crate pulldown_cmark as cmark;

mod page;
mod pages;
mod section;
mod sectioner;
mod span;

use cmark::*;
use pdf_canvas::{BuiltinFont, Pdf};
use std::fs::File;
use std::io::Read;
use std::io::Result;

use pages::Pages;
use sectioner::Sectioner;
use span::Span;

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

pub fn run(output_file: &str, markdown_file: &str) -> Result<()> {
    let mut doc = Pdf::create(&output_file)?;

    let mut markdown_file = File::open(markdown_file)?;
    let mut markdown = String::new();
    markdown_file.read_to_string(&mut markdown)?;

    let parser = Parser::new(&markdown);

    let max_width = PAGE_SIZE.0 - MARGIN.0 - MARGIN.0;
    let mut lines = Sectioner::new(max_width);

    for event in parser {
        lines.parse_event(event);
    }

    let sections = lines.get_vec();

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
                    let delta = (span.pos.0 - pos.0, span.pos.1 - pos.1);
                    t.pos(delta.0, delta.1)?;
                    pos = span.pos;

                    match span.span {
                        Span::Text {
                            text,
                            font_type,
                            font_size,
                        } => {
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
        })?;
    }

    doc.finish()?;
    Ok(())
}
