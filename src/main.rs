extern crate pulldown_cmark as cmark;
extern crate printpdf;

use printpdf::*;
use cmark::*;
use std::fs::File;
use std::io::{BufWriter, Read};

const A4_DIMENSIONS: (Mm, Mm) = (Mm(210.0), Mm(297.0));
const DEFAULT_FONT: BuiltinFont = BuiltinFont::TimesRoman;
const BOLD_FONT: BuiltinFont = BuiltinFont::TimesBold;
const ITALIC_FONT: BuiltinFont = BuiltinFont::TimesItalic;
const ITALIC_BOLD_FONT: BuiltinFont = BuiltinFont::TimesBoldItalic;
const MARGIN: (Mm, Mm) = (Mm(20.0), Mm(20.0));

fn main() {
    let (mut doc, page1, layer1) = PdfDocument::new("Rust MD PDF", A4_DIMENSIONS.0, A4_DIMENSIONS.1, "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    let mut markdown_file = File::open("test.md").unwrap();
    let mut markdown = String::new();
    markdown_file.read_to_string(&mut markdown).unwrap();

    let parser = Parser::new(&markdown);

    let normal_font = doc.add_builtin_font(DEFAULT_FONT).unwrap();
    let bold_font = doc.add_builtin_font(BOLD_FONT).unwrap();
    let mut font = &normal_font;

    current_layer.begin_text_section();
    current_layer.set_font(&font, 16);
    current_layer.set_text_cursor(MARGIN.0, A4_DIMENSIONS.1-MARGIN.1);

    let mut bold = false;
    for event in parser {
        match event {
            Event::Start(Tag::Strong) => {
                font = &bold_font;
                current_layer.set_font(font, 16);
            }
            Event::End(Tag::Strong) => {
                font = &normal_font;
                current_layer.set_font(font, 16);
            }
            Event::Text(text) => current_layer.write_text(text, &font),
            _ => {}
        }
    }
    doc.save(&mut BufWriter::new(File::create("test.pdf").unwrap())).unwrap();
}
