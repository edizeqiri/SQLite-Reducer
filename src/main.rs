mod driver;
mod parser;
mod reducer;

use clap::Parser;
use log::*;
use sqlparser::ast::Statement;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};
use std::str::from_utf8;

// ./reducer –query <query-to-minimize –test <an arbitrary-script>
fn main() {
    let (args, pwd) = init();

    let (query, test_path) = read_and_parse_args(args, pwd);

    /*parser::generate_ast(&query)
        .and_then(|ast| Ok(reducer::reduce(ast)))
        .expect("TODO: panic message");*/

    let test_output = driver::test_query(
        test_path,
        query
    );

    info!("Test output: {:?}", test_output);
}


fn read_and_parse_args(args: Cli, pwd: PathBuf) -> (String, PathBuf) {
    let query_path = pwd.join(args.query);

    let query = fs::read_to_string(query_path)
        .unwrap()
        .replace('\n', "");

    (query, pwd.join(args.test))
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
