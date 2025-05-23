mod driver;
mod parser;
mod reducer;

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

    let ast = parser::generate_ast(&query).and_then(|ast| Ok(reducer::reduce(ast)));
    
    let oracle = driver::init_query(test_path.clone(), query.clone());
    info!("Init output: {:?}", oracle);

    let test_reduce = driver::test_query(test_path, query, oracle?);
    info!("Test output: {:?}", test_reduce);
    
    Ok(())
}

fn read_and_parse_args(args: Cli, pwd: PathBuf) -> (String, PathBuf) {
    let query_path = pwd.join(args.query);

    let query = fs::read_to_string(query_path).unwrap().replace('\n', "");

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
