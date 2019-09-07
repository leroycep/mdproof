/*
  This Source Code Form is subject to the terms of the Mozilla Public
  License, v. 2.0. If a copy of the MPL was not distributed with this
  file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use crate::resources::Resources;
use crate::style::{Class, Style};
use crate::Config;
use printpdf::Pt;
use pulldown_cmark::{CowStr, InlineStr};
use rusttype::{Font, Scale};

pub fn width_of_text(resources: &Resources, style: &Style, text: &str) -> Pt {
    let font = font_from_style(resources, style);
    let scale = scale_from_style(resources.get_config(), style);
    let units_per_em = font.units_per_em() as f64;
    let glyph_space_width: f64 = font
        .glyphs_for(text.chars())
        .map(|g| {
            g.standalone()
                .get_data()
                .map(|data| data.unit_h_metrics.advance_width as f64)
                .unwrap()
        })
        .sum();
    Pt(glyph_space_width * scale.x as f64 / units_per_em)
}

pub fn font_height(resources: &Resources, style: &Style) -> Pt {
    let font = font_from_style(resources, style);
    let scale = scale_from_style(resources.get_config(), style);
    let v_metrics = font.v_metrics(scale);
    let height = (v_metrics.ascent - v_metrics.descent + v_metrics.line_gap) as f64;
    Pt(height)
}

pub fn font_from_style<'res>(resources: &'res Resources, style: &Style) -> &'res Font<'res> {
    let config = resources.get_config();
    let strong = style.contains(&Class::Strong);
    let emphasis = style.contains(&Class::Emphasis);

    if style.contains(&Class::Code) {
        resources
            .get_font(&config.mono_font)
            .expect("All fonts should be loaded, or program should've quit")
    } else if strong && emphasis {
        resources
            .get_font(&config.bold_italic_font)
            .expect("All fonts should be loaded, or program should've quit")
    } else if strong {
        resources
            .get_font(&config.bold_font)
            .expect("All fonts should be loaded, or program should've quit")
    } else if emphasis {
        resources
            .get_font(&config.italic_font)
            .expect("All fonts should be loaded, or program should've quit")
    } else {
        resources
            .get_font(&config.default_font)
            .expect("All fonts should be loaded, or program should've quit")
    }
}

pub fn scale_from_style(config: &Config, style: &Style) -> Scale {
    if style.contains(&Class::Heading(4)) {
        config.h4_font_size
    } else if style.contains(&Class::Heading(3)) {
        config.h3_font_size
    } else if style.contains(&Class::Heading(2)) {
        config.h2_font_size
    } else if style.contains(&Class::Heading(1)) {
        config.h1_font_size
    } else {
        config.default_font_size
    }
}

pub fn slice_cow_from_idx<'c>(text: &CowStr<'c>, idx: usize) -> CowStr<'c> {
    match text {
        CowStr::Boxed(string) => CowStr::from(String::from(&string[idx..])),
        CowStr::Borrowed(stringref) => CowStr::Borrowed(&stringref[idx..]),
        CowStr::Inlined(string) => match InlineStr::try_from_str(&string[idx..]) {
            Ok(inlined) => CowStr::Inlined(inlined),
            Err(_) => CowStr::from(String::from(&string[idx..])),
        },
    }
}

pub fn slice_cow_till_idx<'c>(text: &CowStr<'c>, idx: usize) -> CowStr<'c> {
    match text {
        CowStr::Boxed(string) => CowStr::from(String::from(&string[..idx])),
        CowStr::Borrowed(stringref) => CowStr::Borrowed(&stringref[..idx]),
        CowStr::Inlined(string) => match InlineStr::try_from_str(&string[..idx]) {
            Ok(inlined) => CowStr::Inlined(inlined),
            Err(_) => CowStr::from(String::from(&string[..idx])),
        },
    }
}
