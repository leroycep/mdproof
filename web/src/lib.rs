#[macro_use]
extern crate seed;
use seed::prelude::*;

// Model

pub struct Model {
    pub clicks: i128,
    pub markdown: String,
    pdf: Option<PdfBlob>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            clicks: 0,
            markdown: String::new(),
            pdf: None,
        }
    }
}

// Update

#[derive(Clone)]
pub enum Msg {
    Click,
    UpdateMarkdown(String),
}

fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<Msg>) {
    use Msg::*;
    match msg {
        Click => {
            model.clicks += 1;
        }
        UpdateMarkdown(text) => {
            model.markdown = text;
            let config = mdproof_core::Config::default();
            if let Ok(pdf_ref) = mdproof_core::markdown_to_pdf(&model.markdown, &config) {
                let mut output = std::io::BufWriter::new(Vec::new());
                match pdf_ref.save(&mut output) {
                    Err(e) => {
                        log!(e);
                    }
                    Ok(_) => match output.into_inner() {
                        Ok(buffer) => model.pdf = PdfBlob::new(&buffer),
                        Err(e) => log!(e),
                    },
                }
            }
        }
    }
}

// View

fn view(model: &Model) -> impl View<Msg> {
    div![
        textarea![
            attrs! {At::Value => model.markdown},
            input_ev(Ev::Input, Msg::UpdateMarkdown)
        ],
        if let Some(pdf_blob) = &model.pdf {
            p![a!["Download PDF", attrs! {At::Href => pdf_blob.blob_url}]]
        } else {
            seed::empty()
        }
    ]
}

#[wasm_bindgen(start)]
pub fn render() {
    seed::App::build(|_, _| Model::default(), update, view)
        .finish()
        .run();
}

/// The model of the graph screen. It holds data like what part number the user
/// wants to be graphed.
#[derive(Debug, Clone)]
pub struct PdfBlob {
    blob_url: String,
}

impl PdfBlob {
    pub fn new(array: &[u8]) -> Option<Self> {
        let js_buffer = js_sys::Uint8Array::from(array);
        let js_array = js_sys::Array::of1(&js_buffer);
        let mut blob_props = web_sys::BlobPropertyBag::new();
        blob_props
            .type_("application/pdf")
            .endings(web_sys::EndingTypes::Transparent);

        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&js_array, &blob_props)
            .expect("failed to create blog of svg");
        web_sys::Url::create_object_url_with_blob(&blob)
            .ok()
            .map(|blob_url| Self { blob_url })
    }
}

impl Drop for PdfBlob {
    fn drop(&mut self) {
        // Revoke graph download link
        let _ = web_sys::Url::revoke_object_url(&self.blob_url);
    }
}
