use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "mdbrowser",
    about = "A small tool for browsing markdown files."
)]
pub struct CommandLineArgs {
    /// The listening address to bind.
    #[structopt(
        default_value = "0.0.0.0:8080",
        short = "l",
        long = "listening_address"
    )]
    pub listening_address: String,

    /// Change to directory, default to current directory.
    #[structopt(
        parse(from_os_str),
        short = "C",
        long = "directory",
        default_value = "./"
    )]
    pub directory: PathBuf,
}
