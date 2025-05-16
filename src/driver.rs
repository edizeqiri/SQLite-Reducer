use std::path::PathBuf;
use std::process::Command;
use log::info;
use sqlparser::ast::Query;

pub fn test_query(test: PathBuf, reduced_query_file: PathBuf) -> bool {
    let output = Command::new(test)
        .arg(reduced_query_file)
        .output()
        .expect("failed to execute process");

    info!("{:?}", output);
    true
}

fn save_query(query: String) {
    
}