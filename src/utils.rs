use clap::Parser;
use log::*;
use std::path::PathBuf;
use std::{env, fs};
use std::time::Duration;

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

pub fn print_result(orig_query: &String, reduced_query: &String, elapsed_time: Duration) -> Result<(), Box<dyn std::error::Error>> {
    // orig-query&reduced-query&orig-num-stmt&reduced-num-stmt&orig-token&reduced-token&time-taken

    let orig_num_stmt = orig_query.chars().filter(|&c| c == ';').count();
    let reduced_num_stmt = reduced_query.chars().filter(|&c| c == ';').count();

    let orig_num_token = orig_query.split_whitespace().count();
    let reduced_num_token = reduced_query.split_whitespace().count();

    let time_taken = elapsed_time.as_secs_f64() * 1000.0; // in ms

    let output = format!(
        "{}&{}&{}&{}&{}&{}&{}",
        orig_query,
        reduced_query,
        orig_num_stmt,
        reduced_num_stmt,
        orig_num_token,
        reduced_num_token,
        time_taken
    );

    warn!("[ANALYSIS] {:?}", &output);
    write_output_to_file(output, "src/resources/result.csv".into());

    Ok(())
}

pub(crate) fn read_and_parse_args(args: Cli, pwd: PathBuf) -> (String, PathBuf) {
    let query_path = pwd.join(args.query);
    let query = fs::read_to_string(&query_path)
        .expect(&format!("Failed to read query path: {:?}", query_path));

    warn!("[ANALYSIS] QUERY PATH: {:?}[END ANALYSIS]", query_path);
    warn!(
        "[ANALYSIS] ORIGINAL QUERY: {:?}[END ANALYSIS]",
        query.replace(";;", ";").replace("\n", " ")
    );
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
