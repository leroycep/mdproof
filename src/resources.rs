use failure::Error;
use image::{self, DynamicImage};
use printpdf::{IndirectFontRef, PdfDocumentReference};
use rusttype::Font;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct Resources<'doc> {
    doc: &'doc PdfDocumentReference,
    root_path: PathBuf,
    images: BTreeMap<PathBuf, DynamicImage>,
    fonts: BTreeMap<PathBuf, FontResource>,
}

pub struct FontResource {
    pdf_font_ref: IndirectFontRef,
    rusttype_font: Font<'static>,
}

impl<'doc> Resources<'doc> {
    pub fn new(doc: &'doc PdfDocumentReference, root_path: PathBuf) -> Self {
        Self {
            doc: doc,
            root_path: root_path,
            images: BTreeMap::new(),
            fonts: BTreeMap::new(),
        }
    }

    pub fn load_image<P: AsRef<Path>>(&mut self, path: P) -> Result<&DynamicImage, Error> {
        let filename = self.root_path.join(path);
        if self.images.contains_key(&filename) {
            let image = self
                .images
                .get(&filename)
                .expect("BTreeMap said it contained the key");
            Ok(image)
        } else {
            let image = image::open(&filename)?;
            self.images.insert(filename.clone(), image);
            let image = self
                .images
                .get(&filename)
                .expect("I just inserted the key into the map");
            Ok(image)
        }
    }

    pub fn get_font(&mut self, path: &str) -> Result<&FontResource, Error> {
        let filename = self.root_path.join(path);
        if self.fonts.contains_key(&filename) {
            let font = self
                .fonts
                .get(&filename)
                .expect("BTreeMap said it contained the key");
            Ok(font)
        } else {
            let mut font_data = Vec::new();
            let mut font_file = std::fs::File::open(&filename)?;
            font_file.read_to_end(&mut font_data)?;

            let pdf_font_ref = {
                let data_cursor = std::io::Cursor::new(&font_data);
                self.doc
                    .add_external_font(data_cursor)
                    .map_err(|_e| format_err!("Failed to add font to PDF"))?
            };

            let rusttype_font = Font::from_bytes(font_data)?;

            let resource = FontResource {
                rusttype_font,
                pdf_font_ref,
            };

            self.fonts.insert(filename.clone(), resource);
            let font_resource = self
                .fonts
                .get(&filename)
                .expect("I just inserted the key into the map");
            Ok(font_resource)
        }
    }
}
