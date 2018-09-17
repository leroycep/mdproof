use failure::Error;
use image::{self, DynamicImage};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub struct Resources {
    root_path: PathBuf,
    images: BTreeMap<PathBuf, DynamicImage>,
}

impl Resources {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            root_path: root_path,
            images: BTreeMap::new(),
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
}
