extern crate pulldown_cmark as cmark;
extern crate pdf_canvas;

use pdf_canvas::{Pdf, BuiltinFont, FontSource};
use cmark::*;
use std::fs::File;
use std::io::Read;

/// PAGE_SIZE is the size of a sheet of A4 paper in pt
const PAGE_SIZE: (f32, f32) = (595.0, 842.0);
const MARGIN: (f32, f32) = (20.0, 20.0);
const DEFAULT_FONT: BuiltinFont = BuiltinFont::Times_Roman;
const DEFAULT_FONT_SIZE: f32 = 12.0;
const BOLD_FONT: BuiltinFont = BuiltinFont::Times_Bold;
const ITALIC_FONT: BuiltinFont = BuiltinFont::Times_Italic;

const DEFAULT_OUTPUT_FILENAME: &str = "test.pdf";

fn main() {
    let mut doc = Pdf::create(DEFAULT_OUTPUT_FILENAME).unwrap();

    let mut markdown_file = File::open("test.md").unwrap();
    let mut markdown = String::new();
    markdown_file.read_to_string(&mut markdown).unwrap();

    let parser = Parser::new(&markdown);

    doc.render_page(PAGE_SIZE.0, PAGE_SIZE.1, |canvas| {
        let font = canvas.get_font(DEFAULT_FONT);
        let bold = canvas.get_font(BOLD_FONT);
        let italic = canvas.get_font(ITALIC_FONT);
        canvas.text(|t| {
            t.set_font(&font, DEFAULT_FONT_SIZE)?;
            t.set_leading(18.0)?;
            t.pos(MARGIN.0, PAGE_SIZE.1-MARGIN.1)?;

            for event in parser {
                match event {
                    Event::Start(Tag::Strong) => t.set_font(&bold, DEFAULT_FONT_SIZE)?,
                    Event::End(Tag::Strong) => t.set_font(&font, DEFAULT_FONT_SIZE)?,
                    Event::Start(Tag::Emphasis) => t.set_font(&italic, DEFAULT_FONT_SIZE)?,
                    Event::End(Tag::Emphasis) => t.set_font(&font, DEFAULT_FONT_SIZE)?,

                    Event::Text(text) => t.show(&text)?,

                    Event::Start(Tag::Paragraph) => t.show_line(" ")?,
                    Event::End(Tag::Paragraph) => t.show_line(" ")?,
                    Event::HardBreak =>  t.show_line(" ")?,
                    Event::SoftBreak => t.show_line("")?,
                    _ => {}
                }
            }
            Ok(())
        })
    }).unwrap();

    doc.finish().unwrap();
}
