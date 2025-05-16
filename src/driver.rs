use log::info;
use sqlparser::ast::Query;
use std::path::PathBuf;
use std::process::Command;

pub fn test_query(test: PathBuf, reduced_query_file: PathBuf) -> bool {
    let output = Command::new(test)
        .arg(reduced_query_file)
        .output()
        .expect("failed to execute process");

    info!("{:?}", output);
    true
}

fn save_query(query: String) {}
