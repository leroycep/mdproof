use printpdf::{Mm, Pt};
use rusttype::{Font, Scale};
use util::width_of_text;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum FontType {
    Regular,
    Bold,
    Italic,
    BoldItalic,
    Mono,
}

impl FontType {
    pub fn mono(&self) -> Self {
        ::span::FontType::Mono
    }

    pub fn unmono(&self) -> Self {
        ::span::FontType::Regular
    }

    pub fn bold(&self) -> Self {
        use span::FontType::*;
        match self {
            Regular => Bold,
            Italic => BoldItalic,
            ft => *ft,
        }
    }

    pub fn unbold(&self) -> Self {
        use span::FontType::*;
        match self {
            Bold => Regular,
            BoldItalic => Italic,
            ft => *ft,
        }
    }

    pub fn italic(&self) -> Self {
        use span::FontType::*;
        match self {
            Regular => Italic,
            Bold => BoldItalic,
            ft => *ft,
        }
    }

    pub fn unitalic(&self) -> Self {
        use span::FontType::*;
        match self {
            Italic => Regular,
            BoldItalic => Bold,
            ft => *ft,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Span<'collection> {
    Text {
        text: String,
        font: &'collection Font<'collection>,
        font_type: FontType,
        font_scale: Scale,
    },
}

impl<'collection> Span<'collection> {
    pub fn text(
        text: String,
        font: &'collection Font<'collection>,
        font_type: FontType,
        font_scale: Scale,
    ) -> Self {
        Span::Text {
            text,
            font,
            font_type,
            font_scale,
        }
    }

    pub fn width(&self) -> Mm {
        match self {
            Span::Text {
                text,
                font,
                font_scale,
                ..
            } => width_of_text(&text, &font, *font_scale).into(),
        }
    }

    pub fn height(&self) -> Mm {
        match self {
            Span::Text { font_scale, .. } => Pt(font_scale.y as f64).into(),
        }
    }
}

#[derive(Clone)]
pub struct PositionedSpan<'collection> {
    pub span: Span<'collection>,
    pub pos: (Mm, Mm),
}

impl<'collection> PositionedSpan<'collection> {
    pub fn new(span: Span<'collection>, x: Mm, y: Mm) -> Self {
        let pos = (x, y);
        Self { span, pos }
    }
}
