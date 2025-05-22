use log::info;
use sqlparser::ast::Query;
use std::path::PathBuf;
use std::process::Command;
use std::str::{from_utf8, Utf8Error};

pub fn test_query(test: PathBuf, reduced_query_file: PathBuf) -> Result<bool, Utf8Error> {
    let output = Command::new(test)
        .arg(reduced_query_file)
        .output()
        .expect("failed to execute process");

    from_utf8(&output.stdout).and_then(|out| match out.trim() {
        "0" => Ok(false),
        "1" => Ok(true),
        other => panic!("Expected 0 or 1, got `{}`", other),
    })
}

fn save_query(query: String) {}
