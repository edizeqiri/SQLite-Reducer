use clap::Parser;
use log::*;
use std::path::PathBuf;
use std::time::Duration;
use std::{env, fs, process};

use crate::parser::generate_ast;

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

pub fn print_result(
    query_path: &String,
    orig_query: &String,
    reduced: &Vec<String>,
    elapsed_time: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    // orig-num-stmt&reduced-num-stmt&orig-token&reduced-token&time-taken

    let reduced_query = reduced.join(";") + ";";

    let orig_num_stmt = orig_query.chars().filter(|&c| c == ';').count();
    let reduced_num_stmt = reduced_query.chars().filter(|&c| c == ';').count();

    let orig_num_token = orig_query.split_whitespace().count();
    let reduced_num_token = reduced_query.split_whitespace().count();

    let time_taken = elapsed_time.as_secs_f64() * 1000.0; // in ms

    let (_, query_number) = query_path.rsplit('/').nth(1).unwrap().split_at(5);

    let output = format!(
        "{},{},{},{},{}",
        orig_num_stmt, reduced_num_stmt, orig_num_token, reduced_num_token, time_taken
    );

    warn!("[ANALYSIS] {:?} [END ANALYSIS]", &reduced_query);
    write_output_to_file(
        output,
        format!("src/output/result{}.csv", query_number).into(),
    );

    Ok(())
}

pub(crate) fn read_and_parse_args(args: Cli, pwd: PathBuf) -> (String, PathBuf, String) {
    let query_path = pwd.join(args.query);

    let query = fs::read_to_string(&query_path)
        .expect(&format!("Failed to read query path: {:?}", query_path));

    (
        query,
        pwd.join(args.test),
        query_path.to_string_lossy().to_string(),
    )
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

    if args.reduce.is_some() {
        test_sqlparser(pwd.join(args.reduce.unwrap()));
        process::exit(0);
    }

    (args, pwd)
}

fn test_sqlparser(reduced_file: PathBuf) {
    let queries = fs::read_to_string(&reduced_file);
    let binding = queries.unwrap();
    let query_selection = binding.split_inclusive(";");
    for query in query_selection {
        if let Err(parsed_query) = generate_ast(query) {
            warn!("{}", parsed_query);
        }
    }
}

#[derive(Parser)]
pub struct Cli {
    #[arg(long)]
    query: PathBuf,
    #[arg(long)]
    test: PathBuf,
    #[arg(long, value_name = "PATH", required_unless_present = "query")]
    reduce: Option<PathBuf>,
}
