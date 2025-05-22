use log::info;
use sqlparser::ast::Query;
use std::path::PathBuf;
use std::process::Command;
use std::str::from_utf8;

pub fn test_query(test: PathBuf, reduced_query_file: PathBuf) -> bool {
    let output = Command::new(test)
        .arg(reduced_query_file)
        .output()
        .expect("failed to execute process");

    let script_output = from_utf8(&output.stdout);

    match &script_output {
        Ok("0") => false,
        Ok("1") => true,
        Ok(other) => panic!("Expected 0 or 1. But got: {other}"),
        Err(e) => panic!("Couldn't read output from terminal. Error:\n{e}")
    }

}

fn save_query(query: String) {}
