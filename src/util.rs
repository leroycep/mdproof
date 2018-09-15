use printpdf::Pt;
use rusttype::{Font, Scale};

pub fn width_of_text(text: &str, font: &Font, scale: Scale) -> Pt {
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
