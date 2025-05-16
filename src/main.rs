use clap::Parser;
use log::*;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

// ./reducer –query <query-to-minimize –test <an arbitrary-script>
fn main() {
    let (args, pwd) = init();

    let (querys, test_path) = read_and_parse_args(args, pwd);
    test_query(
        test_path,
        "queries/query1/original_test.sql".parse().unwrap(),
    )
}

fn test_query(test: PathBuf, reduced_query_file: PathBuf) {
    let output = Command::new(test)
        .arg(reduced_query_file)
        .output()
        .expect("failed to execute process");

    info!("{:?}", output)
}

fn read_and_parse_args(args: Cli, pwd: PathBuf) -> (String, PathBuf) {
    let query_path = pwd.join(args.query);
    info!("{:?}", query_path);
    let query = fs::read_to_string(query_path)
        .expect("Should have been able to read the query file: `query_path`");

    info!("Query is:\n{query}");

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
