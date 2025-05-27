mod delta_debug;
mod driver;
mod parser;
mod reducer;
mod transformation;
mod utils;

use crate::utils::vec_statement_to_string;
use clap::ArgAction::Set;
use clap::Parser;
use log::*;
use sqlparser::ast::Statement;
use std::path::PathBuf;
use std::process::Command;
use std::str::from_utf8;
use std::{env, fs};

// ./reducer –query <query-to-minimize –test <an arbitrary-script>
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (args, pwd) = init();

    let (query, test_path) = read_and_parse_args(args, pwd);
    driver::init_query(&query, test_path)?;

    let ast = parser::generate_ast(&query)
        .map(reducer::reduce)
        .and_then(|ast| Ok(vec_statement_to_string(&ast.unwrap(), "\n")));
    info!("ast: {:?}", ast.unwrap());
    Ok(())
}

fn read_and_parse_args(args: Cli, pwd: PathBuf) -> (String, PathBuf) {
    let query_path = pwd.join(args.query);

    let query = fs::read_to_string(query_path).unwrap().replace('\n', "");

    (query, pwd.join(args.test))
}

fn write_output_to_file(content: String, path: PathBuf) {
    fs::write(path, content).unwrap();
}

fn init() -> (Cli, PathBuf) {
    env_logger::init();
    let args = Cli::parse();

    info!("query: {:?}, test: {:?}", args.query, args.test);
    let pwd: PathBuf = env::current_dir().unwrap();
    println!("Current directory: {}", pwd.display());

    (args, pwd)
}

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    query: PathBuf,
    #[arg(long)]
    test: PathBuf,
}
