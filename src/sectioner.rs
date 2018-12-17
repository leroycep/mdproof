use super::Config;
use atomizer::{Atom, BlockTag, Break};
use sizer::{SizedEvent, SizedAtom};
use image::GenericImageView;
use printpdf::Mm;
use resources::Resources;
use section::Section;
use span::Span;
use style::Style;
use util::width_of_text;

pub enum SubsectionType {
    List,
    Quote,
}

pub struct Sectioner<'collection> {
    pub x: Mm,
    lines: Vec<Section>,
    current_line: Vec<Span>,
    current_code_block: Vec<Vec<Span>>,
    max_width: Mm,
    subsection: Option<Box<Sectioner<'collection>>>,
    pub is_code: bool,
    is_alt_text: bool,
    cfg: &'collection Config,
    resources: &'collection Resources,
}

impl<'collection> Sectioner<'collection> {
    pub fn new(max_width: Mm, cfg: &'collection Config, resources: &'collection Resources) -> Self {
        Self {
            x: Mm(0.0),
            lines: Vec::new(),
            current_line: Vec::new(),
            current_code_block: Vec::new(),
            max_width: max_width,
            subsection: None,
            is_code: false,
            is_alt_text: false,
            cfg: cfg,
            resources: resources,
        }
    }

    pub fn parse_event(
        &mut self,
        resources: &Resources,
        event: SizedEvent,
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
            SizedEvent::Break(Break::HorizontalRule) => {
                self.push_section(Section::ThematicBreak)
            }

            SizedEvent::StartBlock(BlockTag::List(_)) => self.new_line(),
            SizedEvent::EndBlock(BlockTag::List(_)) => self.push_space(),

            SizedEvent::StartBlock(BlockTag::ListItem) => {
                self.subsection = Some(Box::new(Sectioner::new(
                    self.max_width - self.cfg.list_indentation,
                    &self.cfg,
                    &self.resources,
                )))
            }
            SizedEvent::EndBlock(BlockTag::ListItem) => return Some(SubsectionType::List),

            SizedEvent::StartBlock(BlockTag::BlockQuote) => {
                self.new_line();
                self.subsection = Some(Box::new(Sectioner::new(
                    self.max_width - self.cfg.quote_indentation,
                    &self.cfg,
                    &self.resources,
                )))
            }
            SizedEvent::EndBlock(BlockTag::BlockQuote) => return Some(SubsectionType::Quote),

            SizedEvent::Break(Break::Word) => {
                if self.x > Mm(0.0) {
                    self.write(" ", &Style::default());
                }
            }

            SizedEvent::Break(Break::Page) => self.push_section(Section::page_break()),

            SizedEvent::SizedAtom(SizedAtom { atom, width, height }) => {
                match atom {
                    Atom::Text { text, style } => {
                        self.write_left_aligned(&text, &style);
                    }

                    Atom::Image { uri } => {
                        let span = Span::image(width, height, uri.into_owned().into());
                        self.push_span(span);
                        self.is_alt_text = true;
                    }
                };
            }

            SizedEvent::StartBlock(BlockTag::CodeBlock) => {
                self.is_code = true;
            }
            SizedEvent::EndBlock(BlockTag::CodeBlock) => {
                let code_block = Section::code_block(self.current_code_block.clone());
                self.push_section(code_block);
                self.current_code_block.clear();

                self.push_space();
                self.is_code = false;
            }

            SizedEvent::Break(Break::Paragraph) => {
                self.new_line();
                self.push_space();
            }

            SizedEvent::Break(Break::Line) => self.new_line(),
        };
        None
    }

    pub fn push_space(&mut self) {
        let spacing = Section::space(self.cfg.section_spacing);
        self.push_section(spacing);
    }

    pub fn push_section(&mut self, section: Section) {
        self.lines.push(section);
    }

    pub fn write_left_aligned(&mut self, text: &str, style: &Style) {
        let width = width_of_text(self.cfg, self.resources, style, text).into();
        if self.x + width > self.max_width {
            self.new_line();
        }

        let span = Span::text(text.to_string(), style.clone());
        self.push_span(span);
    }

    pub fn write(&mut self, text: &str, style: &Style) {
        let span = Span::text(text.into(), style.clone());
        self.push_span(span);
    }

    pub fn push_span(&mut self, span: Span) {
        self.x += span.width(self.cfg, self.resources);
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

    pub fn get_vec(mut self) -> Vec<Section> {
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
