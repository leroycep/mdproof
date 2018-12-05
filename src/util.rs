use printpdf::Pt;
use rusttype::{Font, Scale};
use style::{Class, Style};
use Config;

pub fn width_of_text(config: &Config, style: &Style, text: &str) -> Pt {
    let font = font_from_style(config, style);
    let scale = scale_from_style(config, style);
    let units_per_em = font.units_per_em() as f64;
    let glyph_space_width: f64 = font
        .glyphs_for(text.chars())
        .map(|g| {
            g.standalone()
                .get_data()
                .map(|data| data.unit_h_metrics.advance_width as f64)
                .unwrap()
        }).sum();
    Pt(glyph_space_width * scale.x as f64 / units_per_em)
}

pub fn font_height(config: &Config, style: &Style) -> Pt {
    let font = font_from_style(config, style);
    let scale = scale_from_style(config, style);
    let v_metrics = font.v_metrics(scale);
    let height = (v_metrics.ascent - v_metrics.descent + v_metrics.line_gap) as f64;
    Pt(height)
}

pub fn font_from_style<'cfg>(config: &'cfg Config, style: &Style) -> &'cfg Font<'cfg> {
    let strong = style.contains(&Class::Strong);
    let emphasis = style.contains(&Class::Emphasis);

    if style.contains(&Class::Code) {
        &config.mono_font
    } else if strong && emphasis {
        &config.bold_italic_font
    } else if strong {
        &config.bold_font
    } else if emphasis {
        &config.italic_font
    } else {
        &config.default_font
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
