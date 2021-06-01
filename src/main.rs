use pulldown_cmark::{html, Event, Options, Parser};
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;
use warp::{http::Response, Filter};

mod cli;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let cli::CommandLineArgs {
        listening_address,
        directory,
    } = cli::CommandLineArgs::from_args();

    let directory_copy = directory.clone();
    let directory = Arc::new(directory);

    let routes = warp::get()
        .and(warp::path::param())
        .and(warp::any().map(move || directory.clone()))
        .and_then(handle_markdown)
        .or(warp::fs::dir(directory_copy));

    warp::serve(routes)
        .run(listening_address.parse::<SocketAddr>().unwrap())
        .await;
}

async fn handle_markdown(
    fpath: String,
    directory: Arc<PathBuf>,
) -> Result<impl warp::Reply, warp::Rejection> {
    if fpath.ends_with(".md") {
        let contents =
            fs::read_to_string(directory.to_str().unwrap().to_owned() + "/" + &fpath).unwrap();

        // Set up options and parser. Strikethroughs are not part of the CommonMark standard
        // and we therefore must enable it explicitly.
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(&contents, options);

        // Write to String buffer.
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        let mut result = format!(
            "<!DOCTYPE html>
<html>
<head>
<title>{}</title>
<style>
  body {{
    width: 1024px;
    margin: auto;
  }}
  pre {{
    white-space: pre-wrap;       /* Since CSS 2.1 */
    white-space: -moz-pre-wrap;  /* Mozilla, since 1999 */
    white-space: -pre-wrap;      /* Opera 4-6 */
    white-space: -o-pre-wrap;    /* Opera 7 */
    word-wrap: break-word;       /* Internet Explorer 5.5+ */
}}
</style>
</head>
<body>",
            get_title(&contents)
        );
        result.push_str(&html_output);
        result.push_str("</body>");

        Ok(Response::builder().body(result))
    } else {
        Err(warp::reject())
    }
}

fn get_title(contents: &str) -> String {
    let parser = Parser::new(contents);
    for event in parser {
        match event {
            Event::Text(text) => return text.to_string(),
            _ => continue,
        }
    }

    "".to_owned()
}
