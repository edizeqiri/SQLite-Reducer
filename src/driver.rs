use once_cell::sync::OnceCell;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::str::from_utf8;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct Setup {
    pub test: PathBuf,
    pub oracle: OnceCell<String>,
}

static GLOBAL_SETUP: OnceLock<Setup> = OnceLock::new();
static GLOBAL_TEST_SCRIPT_PATH: OnceCell<PathBuf> = OnceCell::new();
static GLOBAL_EXPECTED_RESULT: OnceCell<String> = OnceCell::new();


pub fn init_test_only(test_value: &PathBuf) {
    let s = Setup {
        test: test_value.into(),
        oracle: OnceCell::new(),
    };
    GLOBAL_SETUP.set(s).expect("GLOBAL was already initialized");
}

pub fn fill_oracle(oracle_value: &str) {
    // this panics if called before init_test_only
    let cell = &GLOBAL_SETUP.get().expect("GLOBAL not init").oracle;
    // set the oracle exactly once
    cell.set(oracle_value.into())
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

pub fn init_query(query: &String, test_script_path: PathBuf) -> Result<_, Box<dyn std::error::Error>> {
    &GLOBAL_TEST_SCRIPT_PATH.set(test_script_path).expect("test script path already initialized");

    let expected_result = from_utf8(&get_output_from_query(query)?.stdout)? // -> &str
                                    .trim() // -> &str
                                    .to_owned(), // -> String
                            );
    info!("Expected result is: {:?}", expected_result);

    &GLOBAL_EXPECTED_RESULT.set(expected_result).expect("test script path already initialized");

    Ok(())
}


fn get_output_from_query(query: &String) -> io::Result<Output> {
    let test_script_path = &GLOBAL_TEST_SCRIPT_PATH.get().expect("We are missing a GLOBAL_TEST_SCRIPT_PATH?!");
    let expected_output = &GLOBAL_EXPECTED_RESULT.get()
    Command::new(&setup.test)
        .arg(query)
        .arg(setup.oracle.get().map(|s| s.as_str()).unwrap_or(""))
        .output()
}
