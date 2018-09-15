extern crate failure;
extern crate printpdf;
extern crate pulldown_cmark as cmark;
extern crate rusttype;
extern crate scraper;

mod page;
mod pages;
mod section;
mod sectioner;
mod span;
mod util;

use cmark::*;
use failure::{Error, ResultExt};
use printpdf::{Mm, PdfDocument};
use rusttype::{Font, Scale};
use std::fs::File;
use std::io::Read;

use pages::Pages;
use sectioner::Sectioner;
use span::Span;

const REGULAR_FONT: &[u8] = include_bytes!("../assets/Noto_Sans/NotoSans-Regular.ttf");
const BOLD_FONT: &[u8] = include_bytes!("../assets/Noto_Sans/NotoSans-Bold.ttf");
const ITALIC_FONT: &[u8] = include_bytes!("../assets/Noto_Sans/NotoSans-Italic.ttf");
const BOLD_ITALIC_FONT: &[u8] = include_bytes!("../assets/Noto_Sans/NotoSans-BoldItalic.ttf");
const MONO_FONT: &[u8] = include_bytes!("../assets/Inconsolata/Inconsolata-Regular.ttf");

#[derive(Debug)]
pub struct Config {
    page_size: (Mm, Mm),
    margin: (Mm, Mm),
    default_font: Font<'static>,
    bold_font: Font<'static>,
    italic_font: Font<'static>,
    bold_italic_font: Font<'static>,
    mono_font: Font<'static>,

    default_font_size: Scale,
    h1_font_size: Scale,
    h2_font_size: Scale,
    h3_font_size: Scale,
    h4_font_size: Scale,

    line_spacing: f64, // Text height * LINE_SPACING
    list_indentation: Mm,
    quote_indentation: Mm,
}

impl Config {
    pub fn get_font_for_type(&self, font_type: span::FontType) -> &Font<'static> {
        use span::FontType::*;
        match font_type {
            Regular => &self.default_font,
            Bold => &self.bold_font,
            Italic => &self.italic_font,
            BoldItalic => &self.bold_italic_font,
            Mono => &self.mono_font,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            page_size: (Mm(210.0), Mm(297.0)),
            margin: (Mm(20.0), Mm(20.0)),
            default_font: Font::from_bytes(REGULAR_FONT).expect("Static font to work"),
            bold_font: Font::from_bytes(BOLD_FONT).expect("Static font to work"),
            italic_font: Font::from_bytes(ITALIC_FONT).expect("Static font to work"),
            bold_italic_font: Font::from_bytes(BOLD_ITALIC_FONT).expect("Static font to work"),
            mono_font: Font::from_bytes(MONO_FONT).expect("Static font to work"),

            default_font_size: Scale::uniform(12.0),
            h1_font_size: Scale::uniform(32.0),
            h2_font_size: Scale::uniform(28.0),
            h3_font_size: Scale::uniform(20.0),
            h4_font_size: Scale::uniform(16.0),

            line_spacing: 1.75, // Text height * LINE_SPACING
            list_indentation: Mm(20.0),
            quote_indentation: Mm(20.0),
        }
    }
}

pub fn run(output_file: &str, markdown_file: &str, cfg: &Config) -> Result<(), Error> {
    let (doc, mut page_idx, mut layer_idx) =
        PdfDocument::new("TITLE", cfg.page_size.0, cfg.page_size.1, "Layer 1");

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

    let default_font_reader = std::io::Cursor::new(REGULAR_FONT);
    let bold_font_reader = std::io::Cursor::new(BOLD_FONT);
    let italic_font_reader = std::io::Cursor::new(ITALIC_FONT);
    let bold_italic_font_reader = std::io::Cursor::new(BOLD_ITALIC_FONT);
    let mono_font_reader = std::io::Cursor::new(MONO_FONT);

    let regular = doc.add_external_font(default_font_reader)?;
    let bold = doc.add_external_font(bold_font_reader)?;
    let italic = doc.add_external_font(italic_font_reader)?;
    let bold_italic = doc.add_external_font(bold_italic_font_reader)?;
    let mono = doc.add_external_font(mono_font_reader)?;

    let mut is_first_iteration = true;

    for page in pages {
        if !is_first_iteration {
            let (new_page_idx, new_layer_idx) =
                doc.add_page(cfg.page_size.0, cfg.page_size.1, "Layer 1");
            page_idx = new_page_idx;
            layer_idx = new_layer_idx;
        }

        let current_layer = doc.get_page(page_idx).get_layer(layer_idx);
        let mut page = page.into_vec().into_iter().peekable();
        for span in page {
            current_layer.begin_text_section();
            current_layer.set_text_cursor(span.pos.0, span.pos.1);

            match span.span {
                Span::Text {
                    text,
                    font_type,
                    font_scale,
                    ..
                } => {
                    use span::FontType;
                    let font = match font_type {
                        FontType::Regular => &regular,
                        FontType::Bold => &bold,
                        FontType::Italic => &italic,
                        FontType::BoldItalic => &bold_italic,
                        FontType::Mono => &mono,
                    };

                    current_layer.set_font(font, font_scale.y as i64);
                    current_layer.write_text(text, font);
                }
                Span::Rect { width, height } => {
                    use printpdf::{Line, Point};
                    let rect_points = vec![
                        (Point::new(span.pos.0, span.pos.1 + height), false),
                        (Point::new(span.pos.0 + width, span.pos.1 + height), false),
                        (Point::new(span.pos.0 + width, span.pos.1), false),
                        (Point::new(span.pos.0, span.pos.1), false),
                    ];
                    let rect = Line {
                        points: rect_points,
                        is_closed: true,
                        has_fill: true,
                        has_stroke: false,
                        is_clipping_path: false,
                    };
                    current_layer.add_shape(rect);
                }
            }
            current_layer.end_text_section();
        }
        is_first_iteration = false;
    }

    use std::io::BufWriter;
    let out = File::create(output_file).with_context(|_| "Failed to create pdf file")?;
    let mut buf_writer = BufWriter::new(out);
    doc.save(&mut buf_writer)
        .with_context(|_| "Failed to save pdf file")?;
    Ok(())
}
