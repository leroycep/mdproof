use atomizer::{Atom, BlockTag, Break, Event as AtomizerEvent};
use image::GenericImageView;
use printpdf::Mm;
use resources::Resources;
use util::{font_height, width_of_text};

const IMAGE_DPI: f64 = 300.0;
const WIDTH_IMAGE_NOT_FOUND: Mm = Mm(50.0);
const HEIGHT_IMAGE_NOT_FOUND: Mm = Mm(50.0);

pub struct Sizer<'src, 'res, I>
where
    I: Iterator<Item = AtomizerEvent<'src>>,
{
    events: I,
    resources: &'res Resources,
}

#[derive(Debug)]
pub enum SizedEvent<'src> {
    SizedAtom(SizedAtom<'src>),
    Break(Break),
    StartBlock(BlockTag),
    EndBlock(BlockTag),
}

#[derive(Debug)]
pub struct SizedAtom<'src> {
    pub atom: Atom<'src>,
    pub width: Mm,
    pub height: Mm,
}

impl<'src, 'res, I> Iterator for Sizer<'src, 'res, I>
where
    I: Iterator<Item = AtomizerEvent<'src>>,
{
    type Item = SizedEvent<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        let event = match self.events.next() {
            Some(e) => e,
            None => return None,
        };
        match event {
            AtomizerEvent::StartBlock(block_type) => Some(SizedEvent::StartBlock(block_type)),
            AtomizerEvent::EndBlock(block_type) => Some(SizedEvent::EndBlock(block_type)),
            AtomizerEvent::Break(break_type) => Some(SizedEvent::Break(break_type)),

            AtomizerEvent::Atom(Atom::Text { text, style }) => {
                let width = width_of_text(self.resources, &style, &text).into();
                let height = font_height(self.resources, &style).into();

                let sized_atom = SizedAtom {
                    atom: Atom::Text { text, style },
                    width: width,
                    height: height,
                };
                Some(SizedEvent::SizedAtom(sized_atom))
            }

            AtomizerEvent::Atom(Atom::Image { uri }) => {
                // TODO: Use title, and ignore alt-text
                // Or should alt-text always be used?
                if let Some(image) = self.resources.get_image(&uri) {
                    let (w, h) = image.dimensions();
                    let (w, h) = (
                        ::printpdf::Px(w as usize).into_pt(IMAGE_DPI).into(),
                        ::printpdf::Px(h as usize).into_pt(IMAGE_DPI).into(),
                    );
                    let sized_atom = SizedAtom {
                        atom: Atom::Image { uri },
                        width: w,
                        height: h,
                    };
                    Some(SizedEvent::SizedAtom(sized_atom))
                } else {
                    warn!("Couldn't load image: {:?}", uri);
                    let sized_atom = SizedAtom {
                        atom: Atom::Image { uri },
                        width: WIDTH_IMAGE_NOT_FOUND,
                        height: HEIGHT_IMAGE_NOT_FOUND,
                    };
                    Some(SizedEvent::SizedAtom(sized_atom))
                }
            }
        }
    }
}

impl<'src, 'res, I> Sizer<'src, 'res, I>
where
    I: Iterator<Item = AtomizerEvent<'src>>,
{
    pub fn new(events: I, resources: &'res Resources) -> Self {
        Self { events, resources }
    }
}
