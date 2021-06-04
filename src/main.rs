use pulldown_cmark::{html, Event, Options, Parser};
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;
use warp::{http::Response, path::FullPath, Filter};

mod cli;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let cli::CommandLineArgs {
        listening_address,
        directory,
        css_class,
        styles,
        scripts,
    } = cli::CommandLineArgs::from_args();

    let directory_copy = directory.clone();
    let directory = Arc::new(directory);
    let style_header = get_html_style_header(&styles);
    let script_header = get_html_script_header(&scripts);

    let routes = warp::get()
        .and(warp::path::full())
        .and(warp::any().map(move || directory.clone()))
        .and(warp::any().map(move || css_class.clone()))
        .and(warp::any().map(move || style_header.clone()))
        .and(warp::any().map(move || script_header.clone()))
        .and_then(handle_markdown)
        .or(warp::fs::dir(directory_copy));

    warp::serve(routes)
        .run(listening_address.parse::<SocketAddr>().unwrap())
        .await;
}

async fn handle_markdown(
    fpath: FullPath,
    directory: Arc<PathBuf>,
    css_class: String,
    style_header: String,
    script_header: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    if fpath.as_str().ends_with(".md") {
        let contents =
            fs::read_to_string(directory.to_str().unwrap().to_owned() + "/" + &fpath.as_str())
                .unwrap();

        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(&contents, options);

        let mut md_output = String::new();
        html::push_html(&mut md_output, parser);

        let mut result = format!(
            "<!DOCTYPE html>
<html>
<head>
<meta charset=\"UTF-8\">
<title>{}</title>
{}
{}
</head>
<body>",
            get_title(&contents),
            style_header,
            script_header
        );
        result.push_str(&format!("<div class=\"{}\">", css_class));
        result.push_str(&md_output);
        result.push_str("</div>");
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

fn get_html_style_header(style_links: &Option<Vec<String>>) -> String {
    style_links
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .filter(|&s| !s.is_empty())
        .map(|s| {
            if s.contains("{") {
                format!("<style>{}</style>\n", s)
            } else {
                format!("<link rel=\"stylesheet\" href=\"{}\">\n", s)
            }
        })
        .fold(String::from(""), |r, s| r + &s)
}

fn get_html_script_header(script_links: &Option<Vec<String>>) -> String {
    script_links
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .filter(|&s| !s.is_empty())
        .map(|s| {
            if s.contains(";") {
                format!("<script type=\"text/javascript\">{}</script>", s)
            } else {
                format!("<script type=\"text/javascript\" src=\"{}\"></script>", s)
            }
        })
        .fold(String::from(""), |r, s| r + &s)
}
