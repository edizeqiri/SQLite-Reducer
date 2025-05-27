use clap::Parser;
use log::info;
use std::path::PathBuf;
use std::{env, fs};

pub fn vec_statement_to_string<T>(
    vector: &Vec<T>,
    separator: &str,
) -> Result<String, Box<dyn std::error::Error>>
where
    T: ToString + PartialEq,
{
    Ok(vector
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(separator)
        + separator)
}

pub fn read_and_parse_args(args: Cli, pwd: PathBuf) -> (String, PathBuf) {
    let query_path = pwd.join(args.query);

    let query = fs::read_to_string(query_path).unwrap().replace('\n', "");

    (query, pwd.join(args.test))
}

fn write_output_to_file(content: String, path: PathBuf) {
    fs::write(path, content).unwrap();
}

pub fn init() -> (Cli, PathBuf) {
    env_logger::init();
    let args = Cli::parse();

    info!("query: {:?}, test: {:?}", args.query, args.test);
    let pwd: PathBuf = env::current_dir().unwrap();
    println!("Current directory: {}", pwd.display());

    (args, pwd)
}

#[derive(Parser)]
pub struct Cli {
    #[arg(long)]
    query: PathBuf,
    #[arg(long)]
    test: PathBuf,
}
