# mdbrowser

[![Rust](https://github.com/yaozongyou/mdbrowser/actions/workflows/rust.yml/badge.svg)](https://github.com/yaozongyou/mdbrowser/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/mdbrowser)](https://crates.io/crates/mdbrowser)

This is a simple tool to render markdown docs. You can specify the style with
--style flag.  You can also specify a different listening address with the -l
flag.  Run --help for more options.

## Example

1. Running with GitHub Markdown style:
```bash
./mdbrowser -C /path/to/markdown/directory                                                           \
    --style="https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/4.0.0/github-markdown.css"   \
    --style="body {width: 1024px;margin: 20px auto 20px;}"                                           \
    --style=".toc {margin-bottom: 16px;} .toc-aux {background: #f9f9f9; border: 1px solid #f2f2f2}"  \
    --style=".version {margin-top: -10px; margin-bottom: 16px;}"
```
This will run a local web server on port 8080 that points to your markdown directory.

2. Running with gitiles style:
```bash
./mdbrowser -C /path/to/markdown/directory                                                           \
    --css_class="doc"                                                                                \
    --style="https://chromium.googlesource.com/+static/base.css"                                     \
    --style="https://chromium.googlesource.com/+static/doc.css"                                      \
    --style="body {width: 1024px;margin: 20px auto 20px;}"                                           \
    --style=".toc {margin-bottom: 16px;} .toc-aux {background: #f9f9f9; border: 1px solid #f2f2f2}"  \
    --style=".version {margin-top: -10px; margin-bottom: 16px;}"
```
