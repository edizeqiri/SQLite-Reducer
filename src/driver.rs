use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::str::{from_utf8};
use std::sync::OnceLock;
use once_cell::sync::OnceCell;

#[derive(Debug)]
pub struct Setup {
    pub test: PathBuf,
    pub oracle: OnceCell<String>,
}

static GLOBAL_SETUP: OnceLock<Setup> = OnceLock::new();

pub fn init_for_testing() {
    let s = Setup {
        test:   PathBuf::from("queries/test.sh"),
        oracle: OnceCell::with_value("&Runtime error near line 1: NOT NULL constraint failed: F.p (19)".to_string()),
    };
    GLOBAL_SETUP
        .set(s)
        .expect("GLOBAL was already initialized");
}

pub fn init_test_only(test_value: &PathBuf) {
    let s = Setup {
        test:   test_value.into(),
        oracle: OnceCell::new(),
    };
    GLOBAL_SETUP
        .set(s)
        .expect("GLOBAL was already initialized");
}

pub fn fill_oracle(oracle_value: &str) {
    // this panics if called before init_test_only
    let cell = &GLOBAL_SETUP.get().expect("GLOBAL not init").oracle;
    // set the oracle exactly once
    cell
        .set(oracle_value.into())
        .expect("oracle was already set");
}

pub fn test_query(query: &String) -> Result<bool, Box<dyn std::error::Error>> {
    let output = from_utf8(&get_output_from_query(query)?.stdout)? // -> &str
        .trim()
        .to_owned();

    match output.as_str() {
        "0" => Ok(false),
        "1" => Ok(true),
        other => panic!("Expected 0 or 1, got `{}`", other),
    }
}

pub fn init_query(query: &String) -> Result<String, Box<dyn std::error::Error>> {
    Ok(
        from_utf8(&get_output_from_query(query)?.stdout)? // -> &str
            .trim() // -> &str
            .to_owned(), // -> String
    )
}

fn get_output_from_query(query: &String) -> io::Result<Output> {
    let setup = GLOBAL_SETUP.get().expect("There is no GLOBAL_SETUP initialized.");
    Command::new(&setup.test)
        .arg(query)
        .arg(setup
                 .oracle
                 .get()
                 .map(|s| s.as_str())
                 .unwrap_or(""))
        .output()
}
