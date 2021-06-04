# mdbrowser

This is a simple server for rendering the markdown on port 8080.
You can specify a different listening address with the -l flag.
Run --help for more options.

## Example

Here is an example running mdbrowser on 0.0.0.0:8080 and with some styles and scripts:
```bash
./mdbrowser -C /path/to/markdown/directory -l "0.0.0.0:8080" --style="relative/path/to/your/base/css/file" --style="https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/4.0.0/github-markdown.css" -style="//cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.0.1/build/styles/default.min.css" --style="body {width: 1024px;margin: auto;}" --script="relative/path/to/your/customed/script/file" --script="//cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.0.1/build/highlight.min.js" --script="hljs.highlightAll();" 
```


