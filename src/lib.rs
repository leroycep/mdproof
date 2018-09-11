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

#[derive(Debug, Clone)]
pub struct Config {
    /// PAGE_SIZE is the size of a sheet of A4 paper in pt
    page_size: (f32, f32),
    margin: (f32, f32),
    default_font: BuiltinFont,
    bold_font: BuiltinFont,
    italic_font: BuiltinFont,

    default_font_size: f32,
    h1_font_size: f32,
    h2_font_size: f32,
    h3_font_size: f32,
    h4_font_size: f32,

    line_spacing: f32, // Text height * LINE_SPACING
    list_indentation: f32,
    quote_indentation: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            /// PAGE_SIZE is the size of a sheet of A4 paper in pt
            page_size: (595.0, 842.0),
            margin: (50.0, 50.0),
            default_font: BuiltinFont::Times_Roman,
            bold_font: BuiltinFont::Times_Bold,
            italic_font: BuiltinFont::Times_Italic,

            default_font_size: 12.0,
            h1_font_size: 32.0,
            h2_font_size: 28.0,
            h3_font_size: 20.0,
            h4_font_size: 16.0,

            line_spacing: 1.75, // Text height * LINE_SPACING
            list_indentation: 20.0,
            quote_indentation: 20.0,
        }
    }
}

pub fn run(output_file: &str, markdown_file: &str, cfg: &Config) -> Result<()> {
    let mut doc = Pdf::create(&output_file)?;

    let mut markdown_file = File::open(markdown_file)?;
    let mut markdown = String::new();
    markdown_file.read_to_string(&mut markdown)?;

    let parser = Parser::new(&markdown);

    let max_width = cfg.page_size.0 - cfg.margin.0 * 2.0;
    let mut lines = Sectioner::new(max_width, cfg);

    for event in parser {
        lines.parse_event(event);
    }

    let sections = lines.get_vec();

    let mut pages = Pages::new(cfg);
    pages.render_sections(&sections[..], cfg.margin.0);

    let pages = pages.into_vec();

    for page in pages {
        doc.render_page(cfg.page_size.0, cfg.page_size.1, |canvas| {
            let regular = canvas.get_font(cfg.default_font);
            let bold = canvas.get_font(cfg.bold_font);
            let italic = canvas.get_font(cfg.italic_font);
            let mono = canvas.get_font(BuiltinFont::Courier);
            canvas.text(|t| {
                let mut page = page.into_vec().into_iter().peekable();
                let mut pos = match page.peek() {
                    Some(x) => x.pos.clone(),
                    None => return Ok(()),
                };
                t.set_font(&regular, cfg.default_font_size)?;
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
