use failure::Error;
use image::{self, DynamicImage};
use printpdf::{IndirectFontRef, PdfDocumentReference};
use rusttype::Font;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use {
    DEFAULT_BOLD_FONT, DEFAULT_BOLD_ITALIC_FONT, DEFAULT_ITALIC_FONT, DEFAULT_MONO_FONT,
    DEFAULT_REGULAR_FONT,
};

pub struct Resources {
    root_path: PathBuf,
    images: BTreeMap<PathBuf, DynamicImage>,
    fonts: BTreeMap<PathBuf, Font<'static>>,
}

pub struct FontResource {
    pdf_font_ref: IndirectFontRef,
    rusttype_font: Font<'static>,
}

pub(crate) const REGULAR_FONT: &[u8] = include_bytes!("../assets/Noto_Sans/NotoSans-Regular.ttf");
pub(crate) const BOLD_FONT: &[u8] = include_bytes!("../assets/Noto_Sans/NotoSans-Bold.ttf");
pub(crate) const ITALIC_FONT: &[u8] = include_bytes!("../assets/Noto_Sans/NotoSans-Italic.ttf");
pub(crate) const BOLD_ITALIC_FONT: &[u8] =
    include_bytes!("../assets/Noto_Sans/NotoSans-BoldItalic.ttf");
pub(crate) const MONO_FONT: &[u8] = include_bytes!("../assets/Inconsolata/Inconsolata-Regular.ttf");

impl Resources {
    pub fn new(root_path: PathBuf) -> Self {
        let mut res = Self {
            root_path: root_path,
            images: BTreeMap::new(),
            fonts: BTreeMap::new(),
        };
        res.fonts.insert(
            DEFAULT_REGULAR_FONT.into(),
            Font::from_bytes(REGULAR_FONT).expect("Static font to work"),
        );
        res.fonts.insert(
            DEFAULT_BOLD_FONT.into(),
            Font::from_bytes(BOLD_FONT).expect("Static font to work"),
        );
        res.fonts.insert(
            DEFAULT_ITALIC_FONT.into(),
            Font::from_bytes(ITALIC_FONT).expect("Static font to work"),
        );
        res.fonts.insert(
            DEFAULT_BOLD_ITALIC_FONT.into(),
            Font::from_bytes(BOLD_ITALIC_FONT).expect("Static font to work"),
        );
        res.fonts.insert(
            DEFAULT_MONO_FONT.into(),
            Font::from_bytes(MONO_FONT).expect("Static font to work"),
        );

        res
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

    pub fn get_image<P: AsRef<Path>>(&self, path: P) -> Option<&DynamicImage> {
        let filename = self.root_path.join(path);
        self.images.get(&filename)
    }

    pub fn get_font(&self, path: &str) -> Option<&Font> {
        let filename = self.root_path.join(path);
        self.fonts.get(&filename)
    }

    pub fn get_font_mut(&mut self, path: &str) -> Result<&Font, Error> {
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

            let rusttype_font = Font::from_bytes(font_data)?;

            self.fonts.insert(filename.clone(), rusttype_font);
            let font_resource = self
                .fonts
                .get(&filename)
                .expect("I just inserted the key into the map");
            Ok(font_resource)
        }
    }
}
