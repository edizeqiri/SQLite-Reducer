use log::info;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::str::{from_utf8, Utf8Error};
use sqlparser::ast::helpers::stmt_data_loading::StageLoadSelectItem;
use sqlparser::ast::Query;

#[derive(Clone)]
pub struct Setup {
    pub test: PathBuf,
    pub oracle: String,
}

pub fn test_query(
    setup: Setup,
    query: String,
) -> Result<bool, Box<dyn std::error::Error>> {
    let output = from_utf8(&get_output_from_query(setup, query)?.stdout)? // -> &str
        .trim()
        .to_owned();

    match output.as_str() {
        "0" => Ok(false),
        "1" => Ok(true),
        other => panic!("Expected 0 or 1, got `{}`", other),
    }
}

pub fn init_query(
    setup: Setup,
    query: String
) -> Result<String, Box<dyn std::error::Error>> {
    Ok(
        from_utf8(&get_output_from_query(setup, query)?.stdout)? // -> &str
            .trim() // -> &str
            .to_owned(), // -> String
    )
}

fn get_output_from_query(
    setup: Setup,
    query: String
) -> io::Result<Output> {
    Command::new(setup.test)
        .arg(query)
        .arg(setup.oracle)
        .output()
}
