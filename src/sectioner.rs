/*
  This Source Code Form is subject to the terms of the Mozilla Public
  License, v. 2.0. If a copy of the MPL was not distributed with this
  file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use crate::atomizer::{Atom, BlockTag, Break};
use crate::resources::Resources;
use crate::section::Section;
use crate::sizer::{SizedAtom, SizedEvent};
use crate::span::Span;
use crate::style::Style;
use crate::util::width_of_text;
use crate::Config;
use printpdf::Mm;

pub enum SubsectionType {
    List,
    Quote,
}

pub struct Sectioner<'res> {
    x: Mm,
    lines: Vec<Section>,
    current_line: Vec<Span>,
    current_code_block: Vec<Vec<Span>>,
    min_x: Mm,
    max_x: Mm,
    subsection: Option<Box<Sectioner<'res>>>,
    is_code: bool,
    is_alt_text: bool,
    resources: &'res Resources,
    cfg: &'res Config,
}

impl<'res> Sectioner<'res> {
    pub fn new(min_x: Mm, max_x: Mm, resources: &'res Resources) -> Self {
        Self {
            x: min_x,
            lines: Vec::new(),
            current_line: Vec::new(),
            current_code_block: Vec::new(),
            min_x: min_x,
            max_x: max_x,
            subsection: None,
            is_code: false,
            is_alt_text: false,
            resources: resources,
            cfg: resources.get_config(),
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
            SizedEvent::Break(Break::HorizontalRule) => self.push_section(Section::ThematicBreak),

            SizedEvent::StartBlock(BlockTag::List(_)) => self.new_line(),
            SizedEvent::EndBlock(BlockTag::List(_)) => self.push_space(),

            SizedEvent::StartBlock(BlockTag::ListItem) => {
                self.subsection = Some(Box::new(Sectioner::new(
                    self.min_x + self.cfg.list_indentation,
                    self.max_x,
                    &self.resources,
                )))
            }
            SizedEvent::EndBlock(BlockTag::ListItem) => return Some(SubsectionType::List),

            SizedEvent::StartBlock(BlockTag::BlockQuote) => {
                self.new_line();
                self.subsection = Some(Box::new(Sectioner::new(
                    self.min_x + self.cfg.quote_indentation,
                    self.max_x,
                    &self.resources,
                )))
            }
            SizedEvent::EndBlock(BlockTag::BlockQuote) => return Some(SubsectionType::Quote),

            SizedEvent::Break(Break::Word) => {
                if self.x > self.min_x {
                    self.write(" ", &Style::default());
                }
            }

            SizedEvent::Break(Break::Page) => self.push_section(Section::page_break()),

            SizedEvent::SizedAtom(SizedAtom {
                atom,
                width,
                height,
            }) => {
                match atom {
                    Atom::Text { text, style } => {
                        self.write_left_aligned(&text, &style);
                    }

                    Atom::Image { uri } => {
                        let span = Span::image(width, height, uri.into_string().into());
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
        let width = width_of_text(self.resources, style, text).into();
        if self.x + width > self.max_x {
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
        self.x += span.width(self.resources);
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
        self.x = self.min_x;
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
