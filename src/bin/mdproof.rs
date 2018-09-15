#[macro_use]
extern crate quicli;
extern crate mdproof;

use quicli::prelude::*;
use std::fs::File;
use std::io::{stdin, BufWriter, Read};
use std::path::Path;

#[derive(Debug, StructOpt)]
struct Cli {
    /// The markdown file to read and render
    markdown_file: String,
    /// Where to save the generated `.pdf` to
    #[structopt(long = "out", short = "o")]
    output_file: Option<String>,

    #[structopt(flatten)]
    verbosity: Verbosity,
}

main!(|args: Cli, log_level: verbosity| {
    let output_path = match args.output_file {
        Some(text) => Path::new(&text).to_path_buf(),
        None => Path::new(&args.markdown_file).with_extension("pdf"),
    };
    let output_path = output_path
        .to_str()
        .ok_or(format_err!("Could not convert output path to string"))?;

    let cfg = mdproof::Config::default();

    let mut markdown = String::new();
    if args.markdown_file == "-" {
        let stdin = stdin();
        let mut stdin = stdin.lock();
        stdin.read_to_string(&mut markdown)?;
    } else {
        let mut markdown_file = File::open(args.markdown_file)?;
        markdown_file.read_to_string(&mut markdown)?;
    }

    let doc = mdproof::markdown_to_pdf(&markdown, &cfg)?;

    let out = File::create(output_path).with_context(|_| "Failed to create pdf file")?;
    let mut buf_writer = BufWriter::new(out);
    doc.save(&mut buf_writer)
        .with_context(|_| "Failed to save pdf file")?;
});
