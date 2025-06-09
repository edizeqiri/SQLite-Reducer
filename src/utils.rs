use clap::Parser;
use log::*;
use regex::Regex;
use std::fmt::format;
use std::path::PathBuf;
use std::str::from_utf8;
use std::time::Duration;
use std::{env, fs, process};

use crate::driver;
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

pub fn get_test_case_location() -> PathBuf {
    let path: PathBuf = env::var("TEST_CASE_LOCATION")
        .map(PathBuf::from) // converts the String into a PathBuf
        .unwrap_or_else(|_| env::current_dir().unwrap().join("query.sql"));
    info!("Path to final query: {:?}", path);
    env::set_var("TEST_CASE_LOCATION", &path);
    path
}

// orig-num-stmt,reduced-num-stmt,orig-token,reduced-token,time-taken
pub fn print_result(
    query_path: &String,
    orig_query: &String,
    reduced: &String,
    elapsed_time: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let reduced_query = reduced;
    let query_num = env::var("SQL_NUMBER").unwrap_or_else(|_| "0".to_string());
    let mut orig_num_stmt = orig_query.chars().filter(|&c| c == ';').count();

    // if it doesn’t already end in a semicolon (ignoring trailing whitespace), bump it by one
    if !orig_query.trim_end().ends_with(';') {
        orig_num_stmt += 1;
    }

    // 1. collapse ";;;" → ";"
    let normalized = {
        // using a simple regex replace requires `regex = "1.5"` in Cargo.toml
        let re = regex::Regex::new(r";+").unwrap();
        re.replace_all(&reduced_query, ";")
    };

    // 2b. split on ';' and ignore empty pieces to get the same count
    let num_statements_alt = normalized
        .split(';')
        .filter(|piece| !piece.trim().is_empty())
        .count();

    let word_re = regex::Regex::new(r"\b\w+\b").unwrap();

    let orig_num_token = word_re.find_iter(orig_query).count();
    let reduced_num_token = word_re.find_iter(&reduced_query).count();
    let time_taken = elapsed_time.as_secs_f64() * 1000.0; // in ms

    let output = format!(
        "{},{},{},{},{}",
        orig_num_stmt, num_statements_alt, orig_num_token, reduced_num_token, time_taken
    );
    warn!("{}", output);
    warn!("[ANALYSIS] {:?} [END ANALYSIS]", &reduced_query);
    write_output_to_file(
        &output,
        &format!("src/output/result{}.csv", query_num).into(),
    );

    write_output_to_file(&reduced_query, &get_test_case_location());
    let _ = save_final_output( &query_num, &reduced_query);

    Ok(())
}

fn save_final_output(query_num: &String, final_query: &String) -> Result<(), Box<dyn std::error::Error>> {
    let test_case_location = driver::TEST_CASE_LOCATION.get().expect("TEST_CASE_LOCATION is not set and default path doesn't work somehow.");

    let binding = driver::get_output_from_query(test_case_location)?;
    let output = from_utf8(&binding.stdout)?;

    let final_output = format!("{:?}\n\n{}", &output, final_query);
    write_output_to_file(&final_output, &format!("src/output/final_output{}.sql", query_num).into());
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

pub fn write_output_to_file(content: &String, path: &PathBuf) {
    fs::write(path, content).expect(&format!("The path is wrong: {:?}", path));
}

pub fn init() -> (Cli, PathBuf) {
    env_logger::init();
    let args = Cli::parse();

    info!("query: {:?}, test: {:?}", args.query, args.test);
    let pwd: PathBuf = env::current_dir().unwrap();
    println!("Current directory: {}", pwd.display());

    if args.reduce.is_some() {
        //test_sqlparser(pwd.join(args.reduce.unwrap()));
        process::exit(0);
    }

    (args, pwd)
}

/* fn test_sqlparser(reduced_file: PathBuf) {
    let queries = fs::read_to_string(&reduced_file);
    let binding = queries.unwrap();
    let query_selection = binding.split_inclusive(";");
    for query in query_selection {
        if let Err(parsed_query) = generate_ast(query) {
            warn!("{}", parsed_query);
        }
    }
} */

#[derive(Parser)]
pub struct Cli {
    #[arg(long)]
    query: PathBuf,
    #[arg(long)]
    test: PathBuf,
    #[arg(long, value_name = "PATH", required_unless_present = "query")]
    reduce: Option<PathBuf>,
}

#[test]
fn test_existing_test_case_location() {
    env::set_var("TEST_CASE_LOCATION", "/hello/world");
    assert_eq!(get_test_case_location().to_str().unwrap(), "/hello/world");
}

#[test]
fn test_inexitent_test_case_location() {
    env::remove_var("TEST_CASE_LOCATION");
    assert_eq!(
        get_test_case_location().to_str().unwrap(),
        "/workspaces/reducer/query.sql"
    );
}
