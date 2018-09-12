#[macro_use]
extern crate quicli;
extern crate mdproof;

use quicli::prelude::*;

#[derive(Debug, StructOpt)]
struct Cli {
    /// The markdown file to read and render
    markdown_file: String,
    /// Where to save the generated `.pdf` to
    #[structopt(long = "out", short = "o")]
    output_file: String,

    #[structopt(flatten)]
    verbosity: Verbosity,
}

main!(|args: Cli, log_level: verbosity| {
    let cfg = mdproof::Config::default();
    mdproof::run(&args.output_file, &args.markdown_file, &cfg)?;
});
