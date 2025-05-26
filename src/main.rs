mod delta_debug;
mod driver;
mod parser;
mod reducer;
mod transformation;

use crate::driver::{fill_oracle, init_test_only, Setup};
use crate::reducer::reduce_statements;
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

    //let ast = parser::generate_ast(&query).and_then(|ast| Ok(reducer::reduce(ast)));

    init_test_only(&test_path);

    let oracle = driver::init_query(&query);
    info!("Init output: {:?}", oracle);

    fill_oracle(&oracle?);

    let test_reduce = driver::test_query(&query);
    info!("Test output: {:?}", test_reduce);

    let reduced_statements = reduce_statements(parser::generate_ast(&query)?);
    
    write_output_to_file(vec_statement_to_string(&reduced_statements?), "src/output/reduced_statements.txt".parse().unwrap());
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

pub fn vec_statement_to_string<T>(vector: &Vec<T>) -> String
where
    T: Clone + ToString + std::cmp::PartialEq,
{
    vector
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(";")
        + ";"
}

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    query: PathBuf,
    #[arg(long)]
    test: PathBuf,
}
