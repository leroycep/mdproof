#[macro_use]
extern crate quicli;
extern crate mdproof;

use quicli::prelude::*;
use std::fs::File;
use std::io::{stdin, BufWriter, Read};
use std::path::Path;

#[derive(Debug, self::StructOpt)]
struct Cli {
    /// The markdown file to read and render. If `-` is passed, the markdown will be
    /// read from stdin.
    markdown_file: String,

    /// The title to put in the PDFs metadata. Defaults to the input file name,
    /// without the extension.
    #[structopt(long = "title")]
    title: Option<String>,

    /// Where to save the generated PDF to.
    #[structopt(long = "out", short = "o")]
    output_file: Option<String>,

    #[structopt(flatten)]
    verbosity: Verbosity,
}

main!(|args: Cli, log_level: verbosity| {
    let markdown_path = Path::new(&args.markdown_file);
    let output_path = match args.output_file {
        Some(text) => Path::new(&text).to_path_buf(),
        None => markdown_path.with_extension("pdf"),
    };
    let output_path = output_path
        .to_str()
        .ok_or(format_err!("Could not convert output path to string"))?;

    let mut cfg = mdproof::Config::default();

    let mut markdown = String::new();
    let mut generated_title = None;
    if args.markdown_file == "-" {
        let stdin = stdin();
        let mut stdin = stdin.lock();
        stdin.read_to_string(&mut markdown)?;
    } else {
        let mut markdown_file = File::open(markdown_path)?;
        markdown_file.read_to_string(&mut markdown)?;

        // Make the PDF title the stem of the markdown file
        if let Some(file_name) = markdown_path.file_stem() {
            generated_title = file_name.to_str().map(|s| s.into());
        }
    }

    cfg.title = match (args.title, generated_title) {
        (Some(t), _) => t,
        (None, Some(t)) => t,
        (None, None) => cfg.title,
    };

    let doc = mdproof::markdown_to_pdf(&markdown, &cfg)?;

    let out = File::create(output_path).with_context(|_| "Failed to create pdf file")?;
    let mut buf_writer = BufWriter::new(out);
    doc.save(&mut buf_writer)
        .map_err(|_e| format_err!("Failed to save pdf file"))?;
});
