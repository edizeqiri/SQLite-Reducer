use log::info;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::str::{from_utf8, Utf8Error};

pub fn test_query(
    test: PathBuf,
    reduced_query: String,
    oracle: String,
) -> Result<bool, Box<dyn std::error::Error>> {
    let output = from_utf8(&get_output_from_query(test, reduced_query, oracle)?.stdout)? // -> &str
        .trim()
        .to_owned();

    match output.as_str() {
        "0" => Ok(false),
        "1" => Ok(true),
        other => panic!("Expected 0 or 1, got `{}`", other),
    }
}

pub fn init_query(
    test: PathBuf,
    reduced_query: String,
) -> Result<String, Box<dyn std::error::Error>> {
    Ok(
        from_utf8(&get_output_from_query(test, reduced_query, "".to_string())?.stdout)? // -> &str
            .trim() // -> &str
            .to_owned(), // -> String
    )
}

fn get_output_from_query(
    test: PathBuf,
    reduced_query: String,
    get_oracle: String,
) -> io::Result<Output> {
    info!("test: {test:?}, reduced_query: {reduced_query:?}, get_oracle: {get_oracle:?}");
    Command::new(test)
        .arg(&reduced_query)
        .arg(get_oracle)
        .output()
}
