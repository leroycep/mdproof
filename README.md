# `mdproof`

```bash
$ cargo run README.md -o README.pdf
```

A command line program that generates PDFs from markdown files, with no
dependency on LaTeX or a headless browser. It's still a work in progress,
and many features are not implemented at all. Use at your own risk.

## Building

```bash
$ git clone https://github.com/Geemili/mdproof
$ cd mdproof
$ cargo run test.md --out=test.pdf
```

## Why?

There are already a plethora of ways to generate PDFs, including LaTeX, headless
browsers, or libreoffice. However, all of these methods pull in massive
dependencies, and can be difficult to set up correctly.

By comparison, this program relies on only a few (direct) dependencies, and can
be compiled into a single executable file.

The goal of this program is to compile markdown to pdf, without stepping through
dependency hell.
