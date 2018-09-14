use super::Config;
use page::Page;
use printpdf::Mm;
use section::Section;
use span::Span;

pub struct Pages<'collection> {
    pages: Vec<Page<'collection>>,
    current_page: Page<'collection>,
    current_y: Mm,
    cfg: &'collection Config,
}

impl<'collection> Pages<'collection> {
    pub fn new(cfg: &'collection Config) -> Self {
        Self {
            cfg: cfg,
            pages: vec![],
            current_page: Page::new(),
            current_y: cfg.page_size.1 - cfg.margin.1,
        }
    }

    fn new_page(&mut self) {
        self.pages.push(self.current_page.clone());
        self.current_page.clear();
        self.current_y = self.cfg.page_size.1 - self.cfg.margin.1;
    }

    pub fn render_sections(&mut self, sections: &[Section<'collection>], start_x: Mm) {
        let min_y = self.cfg.margin.1;
        for section in sections {
            let height = section.min_step();
            let delta_y = height * -self.cfg.line_spacing;
            if self.current_y + delta_y < min_y {
                self.new_page();
            }
            self.current_y += delta_y;
            match section {
                Section::Plain(spans) => {
                    self.current_page
                        .render_spans(&spans, start_x, self.current_y)
                }
                Section::VerticalSpace(_) => {}
                Section::ListItem(ref sections) => {
                    self.current_page.render_spans(
                        &[Span::text(
                            "o".into(),
                            &self.cfg.mono_font,
                            ::span::FontType::Mono,
                            self.cfg.default_font_size,
                        )],
                        start_x,
                        self.current_y,
                    );
                    self.current_y -= delta_y;
                    let list_indentation = self.cfg.list_indentation;
                    self.render_sections(sections, start_x + list_indentation);
                }
                Section::BlockQuote(ref sections) => {
                    self.current_page.render_spans(
                        &[Span::text(
                            "|".into(),
                            &self.cfg.mono_font,
                            ::span::FontType::Mono,
                            self.cfg.default_font_size,
                        )],
                        start_x,
                        self.current_y,
                    );
                    self.current_y -= delta_y;
                    let quote_indentation = self.cfg.quote_indentation;
                    self.render_sections(sections, start_x + quote_indentation);
                }
            }
        }
    }

    pub fn into_vec(mut self) -> Vec<Page<'collection>> {
        self.pages.push(self.current_page);
        self.pages
    }
}
