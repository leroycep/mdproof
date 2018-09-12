use pdf_canvas::{BuiltinFont, FontSource};

#[derive(Clone, Debug)]
pub enum Span {
    Text {
        text: String,
        font_type: BuiltinFont,
        font_size: f32,
    },
}

impl Span {
    pub fn text(text: String, font_type: BuiltinFont, font_size: f32) -> Self {
        Span::Text {
            text,
            font_type,
            font_size,
        }
    }

    pub fn width(&self) -> f32 {
        match self {
            Span::Text {
                text,
                font_type,
                font_size,
            } => font_type.get_width(*font_size, text),
        }
    }

    pub fn height(&self) -> f32 {
        match self {
            Span::Text { font_size, .. } => *font_size,
        }
    }
}

#[derive(Clone)]
pub struct PositionedSpan {
    pub span: Span,
    pub pos: (f32, f32),
}

impl PositionedSpan {
    pub fn new(span: Span, x: f32, y: f32) -> Self {
        let pos = (x, y);
        Self { span, pos }
    }
}
