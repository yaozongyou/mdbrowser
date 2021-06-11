# mdbrowser

This is a simple tool to render markdown docs. You can specify the style with
--style flag.  You can also specify a different listening address with the -l
flag.  Run --help for more options.

## Example

1. Running with GitHub Markdown style:
```bash
./mdbrowser -C /path/to/markdown/directory --style="https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/4.0.0/github-markdown.css" --style="body {width: 1024px;margin: auto;}"
```
This will run a local web server on port 8080 that points to your markdown directory.
