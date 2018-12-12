use super::Config;
use page::Page;
use printpdf::Mm;
use resources::Resources;
use section::Section;
use span::Span;
use style::Class;

pub struct Pages<'collection> {
    pages: Vec<Page>,
    current_page: Page,
    current_y: Mm,
    cfg: &'collection Config,
    resources: &'collection Resources,
}

impl<'collection> Pages<'collection> {
    pub fn new(cfg: &'collection Config, resources: &'collection Resources) -> Self {
        Self {
            cfg: cfg,
            resources: resources,
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

    pub fn render_sections(&mut self, sections: &[Section], start_x: Mm) {
        let min_y = self.cfg.margin.1;
        for section in sections {
            trace!("rendering section: {:?}", section);
            let height = section.min_step(self.cfg, self.resources);
            let delta_y = height * -self.cfg.line_spacing;
            if self.current_y + delta_y < min_y {
                self.new_page();
            }
            self.current_y += delta_y;
            match section {
                Section::Plain(spans) => self.current_page.render_spans(
                    self.cfg,
                    self.resources,
                    &spans,
                    start_x,
                    self.current_y,
                ),
                Section::VerticalSpace(_) => {}
                Section::ThematicBreak => {
                    let r = Span::rect(self.cfg.page_size.0 - self.cfg.margin.0 - start_x, Mm(1.0));
                    self.current_page.render_spans(
                        self.cfg,
                        self.resources,
                        &[r],
                        start_x,
                        self.current_y,
                    );
                }
                Section::PageBreak => self.new_page(),
                Section::ListItem(ref sections) => {
                    let list_x = start_x + self.cfg.list_indentation;
                    let list_point_x = list_x - self.cfg.list_point_offset;
                    self.current_page.render_spans(
                        self.cfg,
                        self.resources,
                        &[Span::text("o".into(), [Class::Code].iter().into())],
                        list_point_x,
                        self.current_y,
                    );
                    self.current_y -= delta_y;
                    self.render_sections(sections, list_x);
                }
                Section::BlockQuote(ref sections) => {
                    self.current_page.render_spans(
                        self.cfg,
                        self.resources,
                        &[Span::text("|".into(), [Class::Code].iter().into())],
                        start_x,
                        self.current_y,
                    );
                    self.current_y -= delta_y;
                    let quote_indentation = self.cfg.quote_indentation;
                    self.render_sections(sections, start_x + quote_indentation);
                }
                Section::CodeBlock(ref lines) => {
                    self.current_y -= delta_y;
                    let sections: Vec<Section> =
                        lines.iter().map(|x| Section::Plain(x.to_vec())).collect();
                    let code_x = start_x + self.cfg.code_indentation;
                    self.render_sections(&sections, code_x);
                }
            }
        }
    }

    pub fn into_vec(mut self) -> Vec<Page> {
        self.pages.push(self.current_page);
        self.pages
    }
}
