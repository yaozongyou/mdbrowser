mod lib;
use lib::ThreadPool;
mod log;
mod util;
use util::split_subsequence;

use pulldown_cmark::{html, Options, Parser};
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::str;

fn main() {
    message!("main start");

    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let mut lines = split_subsequence(&buffer[..], &[b'\r', b'\n']);
    let first_line = lines.next().unwrap();
    let mut cols = split_subsequence(&first_line, b" ");
    cols.next();
    let uri = cols.next().unwrap();

    message!("uri: {:?}", str::from_utf8(uri).unwrap());

    let fpath = &uri[1..];

    let (status_line, filename) = if Path::new(str::from_utf8(fpath).unwrap()).exists() {
        ("HTTP/1.1 200 OK\r\n\r\n", str::from_utf8(fpath).unwrap())
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    // Set up options and parser. Strikethroughs are not part of the CommonMark standard
    // and we therefore must enable it explicitly.
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&contents, options);

    // Write to String buffer.
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let response = format!("{}{}", status_line, html_output);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
