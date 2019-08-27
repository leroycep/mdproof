/*
  This Source Code Form is subject to the terms of the Mozilla Public
  License, v. 2.0. If a copy of the MPL was not distributed with this
  file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use crate::resources::Resources;
use crate::span::{PositionedSpan, Span};
use printpdf::Mm;

#[derive(Clone)]
pub struct Page {
    positioned_spans: Vec<PositionedSpan>,
}

impl Page {
    pub fn new() -> Self {
        Self {
            positioned_spans: vec![],
        }
    }

    pub fn render_spans(
        &mut self,
        resources: &Resources,
        spans: &[Span],
        start_x: Mm,
        start_y: Mm,
    ) {
        let mut x = start_x;
        let y = start_y;
        for span in spans {
            self.positioned_spans
                .push(PositionedSpan::new(span.clone(), x, y));
            x += span.width(resources);
        }
    }

    pub fn clear(&mut self) {
        self.positioned_spans.clear();
    }

    pub fn into_vec(self) -> Vec<PositionedSpan> {
        self.positioned_spans
    }
}
