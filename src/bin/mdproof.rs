#[macro_use]
extern crate quicli;
extern crate mdproof;

use quicli::prelude::*;
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
    let cfg = mdproof::Config::default();
    mdproof::run(
        output_path
            .to_str()
            .ok_or(format_err!("Could not convert path to string"))?,
        &args.markdown_file,
        &cfg,
    )?;
});
