#[macro_use]
extern crate failure;
extern crate image;
extern crate printpdf;
extern crate pulldown_cmark as cmark;
extern crate rusttype;
extern crate scraper;
#[macro_use]
extern crate log;

mod style;
mod page;
mod pages;
mod atomizer;
mod resources;
mod section;
mod sectioner;
mod span;
mod util;

use cmark::*;
use failure::Error;
use printpdf::{Image, Mm, PdfDocument, PdfDocumentReference};
use rusttype::{Font, Scale};

use pages::Pages;
use resources::Resources;
use sectioner::Sectioner;
use span::Span;
use std::path::PathBuf;
use style::Class;

const REGULAR_FONT: &[u8] = include_bytes!("../assets/Noto_Sans/NotoSans-Regular.ttf");
const BOLD_FONT: &[u8] = include_bytes!("../assets/Noto_Sans/NotoSans-Bold.ttf");
const ITALIC_FONT: &[u8] = include_bytes!("../assets/Noto_Sans/NotoSans-Italic.ttf");
const BOLD_ITALIC_FONT: &[u8] = include_bytes!("../assets/Noto_Sans/NotoSans-BoldItalic.ttf");
const MONO_FONT: &[u8] = include_bytes!("../assets/Inconsolata/Inconsolata-Regular.ttf");

#[derive(Debug)]
pub struct Config {
    /// The path from which images will be loaded
    pub resources_directory: PathBuf,

    pub title: String,
    pub first_layer_name: String,

    pub page_size: (Mm, Mm),
    pub margin: (Mm, Mm),
    pub default_font: Font<'static>,
    pub bold_font: Font<'static>,
    pub italic_font: Font<'static>,
    pub bold_italic_font: Font<'static>,
    pub mono_font: Font<'static>,

    pub default_font_size: Scale,
    pub h1_font_size: Scale,
    pub h2_font_size: Scale,
    pub h3_font_size: Scale,
    pub h4_font_size: Scale,

    pub line_spacing: f64, // Text height * LINE_SPACING
    pub list_indentation: Mm,
    pub list_point_offset: Mm,
    pub quote_indentation: Mm,
    /// The horizontal offset of code blocks
    pub code_indentation: Mm,
    /// The vertical space between two sections (paragraphs, lists, etc.)
    pub section_spacing: Mm,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            resources_directory: PathBuf::new(),

            title: "mdproof".into(),
            first_layer_name: "Layer 1".into(),

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

            line_spacing: 1.0, // Text height * LINE_SPACING
            list_indentation: Mm(10.0),
            list_point_offset: Mm(5.0),
            quote_indentation: Mm(20.0),
            code_indentation: Mm(10.0),
            section_spacing: Mm(5.0),
        }
    }
}

pub fn markdown_to_pdf(markdown: &str, cfg: &Config) -> Result<PdfDocumentReference, Error> {
    let (doc, mut page_idx, mut layer_idx) = PdfDocument::new(
        cfg.title.clone(),
        cfg.page_size.0,
        cfg.page_size.1,
        cfg.first_layer_name.clone(),
    );

    let atomizer = atomizer::Atomizer::new(Parser::new(&markdown));

    let max_width = cfg.page_size.0 - cfg.margin.0 * 2.0;
    let mut resources = Resources::new(cfg.resources_directory.clone());
    let mut lines = Sectioner::new(max_width, cfg);

    for event in atomizer {
        lines.parse_event(&mut resources, event);
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

    let regular = doc
        .add_external_font(default_font_reader)
        .map_err(|_e| format_err!("Failed to add font to PDF"))?;
    let bold = doc
        .add_external_font(bold_font_reader)
        .map_err(|_e| format_err!("Failed to add font to PDF"))?;
    let italic = doc
        .add_external_font(italic_font_reader)
        .map_err(|_e| format_err!("Failed to add font to PDF"))?;
    let bold_italic = doc
        .add_external_font(bold_italic_font_reader)
        .map_err(|_e| format_err!("Failed to add font to PDF"))?;
    let mono = doc
        .add_external_font(mono_font_reader)
        .map_err(|_e| format_err!("Failed to add font to PDF"))?;

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
                    style,
                    ..
                } => {
                    // TODO: Abstract this piece of code away. It violates DRY.
                    let strong = style.contains(&Class::Strong);
                    let emphasis = style.contains(&Class::Emphasis);

                    let font = if style.contains(&Class::Code) {
                        &mono
                    } else if  strong && emphasis {
                        &bold_italic
                    } else if strong {
                        &bold
                    } else if emphasis {
                        &italic
                    } else {
                        &regular
                    };

                    let font_scale = util::scale_from_style(&cfg, &style);

                    current_layer.set_font(font, font_scale.y as i64);
                    current_layer.write_text(text, font);
                }
                Span::Image { path, .. } => {
                    let image = Image::from_dynamic_image(resources.load_image(path)?);
                    image.add_to_layer(
                        current_layer.clone(),
                        Some(span.pos.0),
                        Some(span.pos.1),
                        None,
                        None,
                        None,
                        None,
                    );
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

    Ok(doc)
}
