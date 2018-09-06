extern crate pulldown_cmark as cmark;
extern crate printpdf;

use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

const A4_DIMENSIONS: (f64, f64) = (210.0, 297.0);
const DEFAULT_FONT: BuiltinFont = BuiltinFont::TimesRoman;

fn main() {
    let (mut doc, page1, layer1) = PdfDocument::new("Rust MD PDF", Mm(A4_DIMENSIONS.0), Mm(A4_DIMENSIONS.1), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    let text = "Lorem **ipsum**";

    let font = doc.add_builtin_font(DEFAULT_FONT).unwrap();
    current_layer.use_text(text, 48, Mm(10.0), Mm(200.0), &font);
    doc.save(&mut BufWriter::new(File::create("test.pdf").unwrap())).unwrap();
}
