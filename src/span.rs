use printpdf::Mm;
use crate::resources::Resources;
use std::path::PathBuf;
use crate::style::Style;
use crate::util::{font_height, width_of_text};

#[derive(Clone, Debug)]
pub enum Span {
    Text {
        text: String,
        style: Style,
    },
    Image {
        width: Mm,
        height: Mm,
        path: PathBuf,
    },
    Rect {
        width: Mm,
        height: Mm,
    },
}

impl Span {
    pub fn text(text: String, style: Style) -> Self {
        Span::Text { text, style }
    }

    pub fn image(width: Mm, height: Mm, path: PathBuf) -> Self {
        Span::Image {
            width,
            height,
            path,
        }
    }

    pub fn rect(width: Mm, height: Mm) -> Self {
        Span::Rect { width, height }
    }

    pub fn width(&self, resources: &Resources) -> Mm {
        match self {
            Span::Text { text, style, .. } => width_of_text(resources, style, text).into(),
            Span::Image { width, .. } => *width,
            Span::Rect { width, .. } => *width,
        }
    }

    pub fn height(&self, resources: &Resources) -> Mm {
        match self {
            Span::Text { style, .. } => font_height(resources, style).into(),
            Span::Image { height, .. } => *height,
            Span::Rect { height, .. } => *height,
        }
    }
}

#[derive(Clone)]
pub struct PositionedSpan {
    pub span: Span,
    pub pos: (Mm, Mm),
}

impl PositionedSpan {
    pub fn new(span: Span, x: Mm, y: Mm) -> Self {
        let pos = (x, y);
        Self { span, pos }
    }
}
