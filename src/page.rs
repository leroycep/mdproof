use printpdf::Mm;
use span::{PositionedSpan, Span};

#[derive(Clone)]
pub struct Page<'collection> {
    positioned_spans: Vec<PositionedSpan<'collection>>,
}

impl<'collection> Page<'collection> {
    pub fn new() -> Self {
        Self {
            positioned_spans: vec![],
        }
    }

    pub fn render_spans(&mut self, spans: &[Span<'collection>], start_x: Mm, start_y: Mm) {
        let mut x = start_x;
        let y = start_y;
        for span in spans {
            self.positioned_spans
                .push(PositionedSpan::new(span.clone(), x, y));
            x += span.width();
        }
    }

    pub fn clear(&mut self) {
        self.positioned_spans.clear();
    }

    pub fn into_vec(self) -> Vec<PositionedSpan<'collection>> {
        self.positioned_spans
    }
}
