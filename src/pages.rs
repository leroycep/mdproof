use page::Page;
use section::Section;
use span::Span;
use {
    DEFAULT_FONT, DEFAULT_FONT_SIZE, LINE_SPACING, LIST_INDENTATION, MARGIN, PAGE_SIZE,
    QUOTE_INDENTATION,
};

pub struct Pages {
    pages: Vec<Page>,
    current_page: Page,
    current_y: f32,
}

impl Pages {
    pub fn new() -> Self {
        Self {
            pages: vec![],
            current_page: Page::new(),
            current_y: PAGE_SIZE.1 - MARGIN.1,
        }
    }

    fn new_page(&mut self) {
        self.pages.push(self.current_page.clone());
        self.current_page.clear();
        self.current_y = PAGE_SIZE.1 - MARGIN.1;
    }

    pub fn render_sections(&mut self, sections: &[Section], start_x: f32) {
        let min_y = MARGIN.1;
        for section in sections {
            let height = section.min_step();
            let delta_y = -height * LINE_SPACING;
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
                        &[Span::text("o".into(), DEFAULT_FONT, DEFAULT_FONT_SIZE)],
                        start_x,
                        self.current_y,
                    );
                    self.current_y -= delta_y;
                    self.render_sections(sections, start_x + LIST_INDENTATION);
                }
                Section::BlockQuote(ref sections) => {
                    self.current_page.render_spans(
                        &[Span::text("|".into(), DEFAULT_FONT, DEFAULT_FONT_SIZE)],
                        start_x,
                        self.current_y,
                    );
                    self.current_y -= delta_y;
                    self.render_sections(sections, start_x + QUOTE_INDENTATION);
                }
            }
        }
    }

    pub fn into_vec(mut self) -> Vec<Page> {
        self.pages.push(self.current_page);
        self.pages
    }
}
