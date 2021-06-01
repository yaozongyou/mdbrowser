mod lib;
use lib::ThreadPool;
mod log;
mod util;
use util::split_subsequence;
mod cli;
use pulldown_cmark::{html, Options, Parser};
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::path::PathBuf;
use std::str;
use structopt::StructOpt;

fn main() {
    message!("main start");

    let cli::CommandLineArgs {
        listening_address,
        directory,
    } = cli::CommandLineArgs::from_args();

    let listener = TcpListener::bind(listening_address).unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let directory = directory.clone();

        pool.execute(|| {
            handle_connection(stream, directory);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream, mut directory: PathBuf) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let mut lines = split_subsequence(&buffer[..], &[b'\r', b'\n']);
    let first_line = lines.next().unwrap();
    let mut cols = split_subsequence(&first_line, b" ");
    cols.next();
    let uri = cols.next().unwrap();

    message!("uri: {:?}", str::from_utf8(uri).unwrap());

    directory.push(Path::new(str::from_utf8(&uri[1..]).unwrap()));
    message!("directory: {:?}", directory);
    message!("exists: {:?}", directory.as_path().exists());

    let (status_line, filename) = if directory.as_path().exists() {
        ("HTTP/1.1 200 OK\r\n\r\n", directory)
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", PathBuf::from("404.html"))
    };

    message!("filename: {:?}", filename);
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
