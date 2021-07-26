use chrono::prelude::*;
use git2::{Error, Repository, Status};
use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag};
use regex::Regex;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
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

        // add anchors to headings
        let mut heading_level = 0;
        let parser = parser.filter_map(|event| match event {
            Event::Start(Tag::Heading(level @ 1..=6)) => {
                heading_level = level;
                None
            }
            Event::Text(text) => {
                if heading_level != 0 {
                    let anchor = text
                        .clone()
                        .into_string()
                        .trim()
                        .to_lowercase()
                        .replace(" ", "-");
                    let tmp = Event::Html(CowStr::from(format!(
                        "<h{} id=\"{}\">{}",
                        heading_level, anchor, text
                    )))
                    .into();
                    heading_level = 0;
                    return tmp;
                }
                Some(Event::Text(text))
            }
            _ => Some(event),
        });

        let mut md_output = String::new();
        html::push_html(&mut md_output, parser);
        let re = Regex::new(r"(\p{Han})\p{White_Space}(\p{Han})").unwrap();
        let md_output = re.replace_all(&md_output, "$1$2");

        let md_output =
            md_output.replace("<p>[TOC]</p>", get_table_of_contents(&contents).as_str());

        let version = get_file_git_version(
            &directory.to_str().unwrap().to_owned(),
            &fpath.as_str().to_owned(),
        )
        .unwrap_or("".to_owned());

        let re = Regex::new(r"(</h[1-5]>)").unwrap();
        let md_output = re.replace(&md_output, &("$1 ".to_owned() + &version));

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
                format!(
                    "<link type=\"text/css\" rel=\"stylesheet\" href=\"{}\">\n",
                    s
                )
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
            if s.contains("(") {
                format!("<script type=\"text/javascript\">{}</script>", s)
            } else {
                format!("<script type=\"text/javascript\" src=\"{}\"></script>", s)
            }
        })
        .fold(String::from(""), |r, s| r + &s)
}

fn get_toc_list(contents: &str) -> Vec<(u32, String)> {
    let mut toc_list = vec![];

    let mut level = 0;
    let mut in_heading = false;

    let parser = Parser::new(contents);
    for event in parser {
        match event {
            Event::Text(text) => {
                if in_heading {
                    toc_list.push((level, text.into_string()));
                }
                continue;
            }
            Event::Start(Tag::Heading(h)) => {
                in_heading = true;
                level = h;
                continue;
            }
            Event::End(Tag::Heading(_h)) => {
                in_heading = false;
                continue;
            }
            _ => continue,
        }
    }

    toc_list
}

#[derive(Debug)]
struct ContentItem {
    level: u32,
    text: String,
    children: Vec<ContentItem>,
}

fn append_heading(content_items: &mut Vec<ContentItem>, heading: (u32, String)) {
    match content_items.last_mut() {
        Some(last_content_item) if heading.0 > last_content_item.level => {
            append_heading(&mut last_content_item.children, heading);
        }
        _ => {
            content_items.push(ContentItem {
                level: heading.0,
                text: heading.1,
                children: vec![],
            });
        }
    }
}

fn order_toc_list(mut heading_list: Vec<(u32, String)>) -> Vec<ContentItem> {
    let mut content_items = vec![];

    while !heading_list.is_empty() {
        let heading = heading_list.remove(0);
        append_heading(&mut content_items, heading);
    }

    content_items
}

fn render_toc(contents: &Vec<ContentItem>) -> String {
    if contents.is_empty() {
        return "".to_owned();
    }

    let mut str = String::from("<ul>\n");
    for content in contents {
        str.push_str(
            format!(
                "<li><a href=\"#{}\">{}</a></li>\n",
                content.text.trim().to_lowercase().replace(" ", "-"),
                content.text
            )
            .as_str(),
        );
        str.push_str(render_toc(&content.children).as_str());
    }
    str.push_str("</ul>\n");
    str
}

fn get_table_of_contents(contents: &str) -> String {
    let mut toc = String::from("<div class='toc'>\n");

    let toc_list = get_toc_list(contents);
    let contents = order_toc_list(toc_list);

    toc.push_str("<div class=\"toc-aux\">");
    let str = if contents.len() == 1 {
        render_toc(&contents[0].children)
    } else {
        render_toc(&contents)
    };
    toc.push_str(str.as_str());

    toc.push_str("</div>\n");
    toc.push_str("</div>\n");
    toc
}

fn get_file_git_version(repo_dir: &String, fpath: &String) -> Result<String, Error> {
    let fpath = fpath.trim_start_matches('/');
    let repo = Repository::open(repo_dir)?;
    let status = repo.status_file(Path::new(fpath))?;
    let head = repo.head()?;

    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::TIME)?;
    revwalk.push_head()?;
    for commit_id in revwalk {
        let commit_id = commit_id?;
        let commit = repo.find_commit(commit_id)?;
        // Ignore merge commits (2+ parents) because that's what 'git whatchanged' does.
        // Ignore commit with 0 parents (initial commit) because there's nothing to diff against
        if commit.parent_count() == 1 {
            let prev_commit = commit.parent(0)?;
            let tree = commit.tree()?;
            let prev_tree = prev_commit.tree()?;
            let diff = repo.diff_tree_to_tree(Some(&prev_tree), Some(&tree), None)?;
            for delta in diff.deltas() {
                let file_path = delta.new_file().path().unwrap();
                if file_path == Path::new(fpath) {
                    return Ok(format!(
                        "<div class='version'>version: {}{}@{} last modified at {}</div>",
                        &commit.id().to_string()[0..8],
                        if status == Status::CURRENT { "" } else { "+" },
                        head.shorthand().unwrap_or(""),
                        DateTime::<Local>::from(DateTime::<Utc>::from_utc(
                            NaiveDateTime::from_timestamp(commit.time().seconds(), 0),
                            Utc,
                        ))
                        .format("%Y-%m-%d %H:%M:%S")
                    ));
                }
            }
        }
    }

    Ok("".to_owned())
}
