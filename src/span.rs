use printpdf::Mm;
use std::path::PathBuf;
use style::Style;
use util::{font_height, width_of_text};
use Config;

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

    pub fn width(&self, config: &Config) -> Mm {
        match self {
            Span::Text { text, style, .. } => width_of_text(config, &style, &text).into(),
            Span::Image { width, .. } => width.clone(),
            Span::Rect { width, .. } => width.clone(),
        }
    }

    pub fn height(&self, config: &Config) -> Mm {
        match self {
            Span::Text { style, .. } => font_height(config, &style).into(),
            Span::Image { height, .. } => height.clone(),
            Span::Rect { height, .. } => height.clone(),
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
