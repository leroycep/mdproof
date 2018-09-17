use super::Config;
use cmark::{Event, Tag};
use image::GenericImage;
use printpdf::Mm;
use resources::Resources;
use rusttype::Scale;
use section::Section;
use span::{FontType, Span};
use util::width_of_text;

pub enum SubsectionType {
    List,
    Quote,
}

pub struct Sectioner<'collection> {
    pub x: Mm,
    lines: Vec<Section<'collection>>,
    current_line: Vec<Span<'collection>>,
    current_code_block: Vec<Vec<Span<'collection>>>,
    current_font_type: FontType,
    current_scale: Scale,
    max_width: Mm,
    subsection: Option<Box<Sectioner<'collection>>>,
    pub is_code: bool,
    is_alt_text: bool,
    cfg: &'collection Config,
}

impl<'collection> Sectioner<'collection> {
    pub fn new(max_width: Mm, cfg: &'collection Config) -> Self {
        Self {
            x: Mm(0.0),
            lines: Vec::new(),
            current_line: Vec::new(),
            current_code_block: Vec::new(),
            current_font_type: FontType::Regular,
            current_scale: cfg.default_font_size,
            max_width: max_width,
            subsection: None,
            is_code: false,
            is_alt_text: false,
            cfg: cfg,
        }
    }

    pub fn parse_event(
        &mut self,
        resources: &mut Resources,
        event: Event,
    ) -> Option<SubsectionType> {
        if self.subsection.is_some() {
            let mut subsection = self
                .subsection
                .take()
                .expect("Checked if the subsection was `Some`");
            if let Some(sub_type) = subsection.parse_event(resources, event) {
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
            Event::Start(Tag::Strong) => self.current_font_type = self.current_font_type.bold(),
            Event::End(Tag::Strong) => self.current_font_type = self.current_font_type.unbold(),
            Event::Start(Tag::Emphasis) => self.current_font_type = self.current_font_type.italic(),
            Event::End(Tag::Emphasis) => self.current_font_type = self.current_font_type.unitalic(),

            Event::Start(Tag::Rule) => self.push_section(Section::ThematicBreak),
            Event::End(Tag::Rule) => {}

            Event::Start(Tag::Header(size)) => {
                self.current_scale = match size {
                    1 => self.cfg.h1_font_size,
                    2 => self.cfg.h2_font_size,
                    3 => self.cfg.h3_font_size,
                    _ => self.cfg.h4_font_size,
                }
            }
            Event::End(Tag::Header(_)) => {
                self.current_scale = self.cfg.default_font_size;
                self.new_line();
                self.push_space();
            }

            Event::Start(Tag::List(_)) => self.new_line(),
            Event::End(Tag::List(_)) => self.push_space(),

            Event::Start(Tag::Item) => {
                self.subsection = Some(Box::new(Sectioner::new(
                    self.max_width - self.cfg.list_indentation,
                    &self.cfg,
                )))
            }
            Event::End(Tag::Item) => return Some(SubsectionType::List),

            Event::Start(Tag::BlockQuote) => {
                self.new_line();
                self.subsection = Some(Box::new(Sectioner::new(
                    self.max_width - self.cfg.quote_indentation,
                    &self.cfg,
                )))
            }
            Event::End(Tag::BlockQuote) => return Some(SubsectionType::Quote),

            Event::Text(ref _text) if self.is_alt_text => {}

            Event::Text(ref text) if self.is_code => {
                let mut start = 0;
                for (pos, c) in text.char_indices() {
                    if c == '\n' {
                        self.write(&text[start..pos]);
                        self.new_line();
                        start = pos + 1;
                    }
                }
                if start < text.len() {
                    self.write(&text[start..]);
                }
            }
            Event::Text(text) => self.write_left_aligned(&text),

            Event::Html(html) => {
                use scraper::Html;
                let fragment = Html::parse_fragment(&html);

                for value in fragment.tree.values() {
                    let style_option = value.as_element().map(|e| e.attr("style")).unwrap_or(None);
                    match style_option {
                        Some("page-break-after:always;") => {
                            self.push_section(Section::page_break())
                        }
                        _ => {}
                    }
                }
            }

            Event::Start(Tag::Image(url, _title)) => {
                // TODO: Use title, and ignore alt-text
                // Or should alt-text always be used?
                if let Ok(image) = resources.load_image(url.clone().into_owned()) {
                    let (w, h) = image.dimensions();
                    let (w, h) = (
                        ::printpdf::Px(w as usize).into_pt(300.0).into(),
                        ::printpdf::Px(h as usize).into_pt(300.0).into(),
                    );
                    let span = Span::image(w, h, url.into_owned().into());
                    self.push_span(span);
                    self.is_alt_text = true;
                } else {
                    warn!("Couldn't load image: {:?}", url);
                }
            }
            Event::End(Tag::Image(_url, _title)) => {
                self.is_alt_text = false;
            }

            Event::Start(Tag::Code) => self.current_font_type = self.current_font_type.mono(),
            Event::End(Tag::Code) => self.current_font_type = self.current_font_type.unmono(),

            Event::Start(Tag::CodeBlock(_src_type)) => {
                self.is_code = true;
                self.current_font_type = self.current_font_type.mono();
                self.current_scale = self.cfg.default_font_size;
            }
            Event::End(Tag::CodeBlock(_)) => {
                let code_block = Section::code_block(self.current_code_block.clone());
                self.push_section(code_block);
                self.current_code_block.clear();

                self.push_space();
                self.is_code = false;
                self.current_font_type = self.current_font_type.unmono();
            }

            Event::Start(Tag::Paragraph) => {}
            Event::End(Tag::Paragraph) => {
                self.new_line();
                self.push_space();
            }

            Event::SoftBreak => self.write(" "),
            Event::HardBreak => self.new_line(),

            _ => {}
        };
        None
    }

    pub fn push_space(&mut self) {
        let spacing = Section::space(self.cfg.section_spacing);
        self.push_section(spacing);
    }

    pub fn push_section(&mut self, section: Section<'collection>) {
        self.lines.push(section);
    }

    pub fn write_left_aligned(&mut self, text: &str) {
        let current_font = self.cfg.get_font_for_type(self.current_font_type);
        let space_width = width_of_text(" ", current_font, self.current_scale).into();

        let mut buffer = String::new();
        let mut buffer_width = Mm(0.0);
        let mut pos = 0;
        while pos < text.len() {
            let idx = text[pos..]
                .find(char::is_whitespace)
                .unwrap_or(text.len() - pos - 1)
                + pos
                + 1;
            let word = &text[pos..idx];
            pos = idx;
            let word_width = width_of_text(word, current_font, self.current_scale).into();
            if self.x + buffer_width + word_width > self.max_width {
                self.write(&buffer);
                self.new_line();
                buffer.clear();
                buffer_width = Mm(0.0);
            }
            if buffer.len() > 0 {
                buffer.push(' ');
                buffer_width += space_width;
            }
            buffer.push_str(word);
            buffer_width += word_width;
        }
        let span = Span::text(
            buffer,
            current_font,
            self.current_font_type,
            self.current_scale,
        );
        self.push_span(span);
    }

    pub fn write(&mut self, text: &str) {
        let span = Span::text(
            text.into(),
            self.cfg.get_font_for_type(self.current_font_type),
            self.current_font_type,
            self.current_scale,
        );
        self.push_span(span);
    }

    pub fn push_span(&mut self, span: Span<'collection>) {
        self.x += span.width();
        self.current_line.push(span);
    }

    pub fn new_line(&mut self) {
        if self.current_line.len() == 0 {
            return;
        }
        if self.is_code {
            self.current_code_block.push(self.current_line.clone());
        } else {
            self.lines.push(Section::plain(self.current_line.clone()));
        }
        self.current_line.clear();
        self.x = Mm(0.0);
    }

    pub fn get_vec(mut self) -> Vec<Section<'collection>> {
        // Make sure that current_line is put into the output
        if self.current_line.len() != 0 {
            self.lines.push(Section::plain(self.current_line));
        }
        // Check if the last section is a blank-type of section, so that we
        // don't get an extra page at the end of the document
        if self.lines.last().map(|t| t.is_empty()).unwrap_or(false) {
            self.lines.pop();
        }
        self.lines
    }
}
