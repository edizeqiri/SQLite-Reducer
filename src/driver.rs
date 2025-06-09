use log::info;
use once_cell::sync::OnceCell;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Output;
use std::str::from_utf8;
use std::sync::OnceLock;

use crate::utils;
use crate::utils::vec_statement_to_string;

static GLOBAL_TEST_SCRIPT_PATH: OnceCell<PathBuf> = OnceCell::new();
static GLOBAL_EXPECTED_RESULT: OnceCell<String> = OnceCell::new();
pub(crate) static TEST_CASE_LOCATION: OnceCell<PathBuf> = OnceCell::new();

pub fn test_query(query: &String) -> Result<bool, Box<dyn std::error::Error>> {
    let test_case_location = TEST_CASE_LOCATION
        .get()
        .expect("TEST_CASE_LOCATION is not set and default path doesn't work somehow.");

    // write query into the corresponding file
    utils::write_output_to_file(query, test_case_location);
    // `cmd` is now an owned `Command`
    let (output, status) = get_exit_status_from_query();

    match status?.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        Some(other) => panic!("Expected exit code 0 or 1, got `{}`", other),
        None => Err("Process terminated by signal, no exit code available".into()),
    }
}

pub fn init_query(
    query: &PathBuf,
    test_script_path: PathBuf,
    test_case_location: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    GLOBAL_TEST_SCRIPT_PATH
        .set(test_script_path)
        .expect("test script path already initialized");

    let expected_result = from_utf8(&get_output_from_query(query)?.stdout)? // -> &str
        .trim()
        .to_string();

    info!("Expected result is: {:?}", expected_result);

    GLOBAL_EXPECTED_RESULT
        .set(expected_result)
        .expect("test script path already initialized");

    TEST_CASE_LOCATION
        .set(PathBuf::from(test_case_location))
        .expect("where is the test case location");

    Ok(())
}

pub(crate) fn get_output_from_query(query: &PathBuf) -> io::Result<Output> {
    let test_script_path = GLOBAL_TEST_SCRIPT_PATH
        .get()
        .expect("We are missing a GLOBAL_TEST_SCRIPT_PATH?!");

    let out = Command::new(test_script_path).arg(query).arg("").output();
    out
}

fn get_exit_status_from_query() -> (io::Result<Output>, io::Result<ExitStatus>) {
    let test_script_path = GLOBAL_TEST_SCRIPT_PATH
        .get()
        .expect("We are missing a GLOBAL_TEST_SCRIPT_PATH?!");

    let expected_output: &str = GLOBAL_EXPECTED_RESULT
        .get()
        .map(|s| s.as_str())
        .unwrap_or("");

    let test_case_location = TEST_CASE_LOCATION
        .get()
        .expect("TEST_CASE_LOCATION is not set and default path doesn't work somehow.");

    // Build an owned `Command` here:
    let mut binding = Command::new(test_script_path);

    let cmd = binding.arg(&test_case_location).arg(expected_output);

    // Return it by value (not by reference):
    (cmd.output(), cmd.status())
}
