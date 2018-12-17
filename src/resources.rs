use failure::Error;
use image::{self, DynamicImage};
use rusttype::Font;
use std::collections::{BTreeMap, HashSet};
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

pub trait Loader {
    fn queue_font(&mut self, path: &str);
    fn queue_image(&mut self, path: &str);
    fn load_resources(&self) -> (Resources, Vec<Error>);
}

pub struct SimpleLoader {
    root_path: PathBuf,
    queued_images: HashSet<String>,
    queued_fonts: HashSet<String>,
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

    pub fn add_image(&mut self, path: &str, image: DynamicImage) {
        let filename = self.root_path.join(path);
        self.images.insert(filename, image);
    }

    pub fn get_image(&self, path: &str) -> Option<&DynamicImage> {
        let filename = self.root_path.join(path);
        self.images.get(&filename)
    }

    pub fn add_font(&mut self, path: &str, font: Font<'static>) {
        let filename = self.root_path.join(path);
        self.fonts.insert(filename, font);
    }

    pub fn get_font(&self, path: &str) -> Option<&Font> {
        let filename = self.root_path.join(path);
        self.fonts.get(&filename)
    }
}

impl SimpleLoader {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            root_path: root_path,
            queued_images: HashSet::new(),
            queued_fonts: HashSet::new(),
        }
    }

    fn load_font(&self, font: &str) -> Result<Font<'static>, Error> {
        let filename = self.root_path.join(font);

        let mut buffer = Vec::new();
        let mut font_file = std::fs::File::open(&filename)?;
        font_file.read_to_end(&mut buffer)?;

        let font = Font::from_bytes(buffer)?;
        Ok(font)
    }

    fn load_image(&self, image_path: &str) -> Result<DynamicImage, Error> {
        let filename = self.root_path.join(image_path);

        let image = image::open(&filename)?;
        Ok(image)
    }
}

impl Loader for SimpleLoader {
    fn queue_font(&mut self, path: &str) {
        self.queued_fonts.insert(path.to_string());
    }

    fn queue_image(&mut self, path: &str) {
        self.queued_images.insert(path.to_string());
    }

    fn load_resources(&self) -> (Resources, Vec<Error>) {
        let mut res = Resources::new(self.root_path.clone());
        let mut errors = Vec::new();
        for font_name in self.queued_fonts.iter() {
            match self.load_font(font_name) {
                Ok(font) => res.add_font(font_name, font),
                Err(e) => errors.push(e),
            }
        }
        for image_path in self.queued_images.iter() {
            match self.load_image(image_path) {
                Ok(image) => res.add_image(image_path, image),
                Err(e) => errors.push(e),
            }
        }
        (res, errors)
    }
}
