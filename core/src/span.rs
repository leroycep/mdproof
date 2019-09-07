/*
  This Source Code Form is subject to the terms of the Mozilla Public
  License, v. 2.0. If a copy of the MPL was not distributed with this
  file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use crate::resources::Resources;
use crate::style::Style;
use crate::util::{font_height, width_of_text};
use printpdf::Mm;
use std::path::PathBuf;

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
            Span::Text { text, style, .. } => width_of_text(resources, &style, &text).into(),
            Span::Image { width, .. } => width.clone(),
            Span::Rect { width, .. } => width.clone(),
        }
    }

    pub fn height(&self, resources: &Resources) -> Mm {
        match self {
            Span::Text { style, .. } => font_height(resources, &style).into(),
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
