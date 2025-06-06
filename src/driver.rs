use log::info;
use once_cell::sync::OnceCell;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Output;
use std::str::from_utf8;

static GLOBAL_TEST_SCRIPT_PATH: OnceCell<PathBuf> = OnceCell::new();
static GLOBAL_EXPECTED_RESULT: OnceCell<String> = OnceCell::new();

pub fn test_query(query: &String) -> Result<bool, Box<dyn std::error::Error>> {
    let (output, status) = get_exit_status_from_query(query);

    info!("{:?}", output);

    match status?.code() {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        Some(other) => panic!("Expected exit code 0 or 1, got `{}`", other),
        None => Err("Process terminated by signal, no exit code available".into()),
    }
}

pub fn init_query(
    query: &String,
    test_script_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    &GLOBAL_TEST_SCRIPT_PATH
        .set(test_script_path)
        .expect("test script path already initialized");

    let expected_result = from_utf8(&get_output_from_query(query)?.stdout)? // -> &str
        .trim()
        .to_string();

    info!("Expected result is: {:?}", expected_result);

    let _ = &GLOBAL_EXPECTED_RESULT
        .set(expected_result)
        .expect("test script path already initialized");

    Ok(())
}

fn get_output_from_query(query: &String) -> io::Result<Output> {
    let test_script_path = GLOBAL_TEST_SCRIPT_PATH
        .get()
        .expect("We are missing a GLOBAL_TEST_SCRIPT_PATH?!");
    let expected_output: &str = GLOBAL_EXPECTED_RESULT
        .get() // Option<&String>
        .map(|s| s.as_str()) // Option<&str>
        .unwrap_or(""); // &str

    Command::new(test_script_path)
        .arg(query)
        .arg(expected_output)
        .output()
}

fn get_exit_status_from_query(query: &String) -> (io::Result<Output>, io::Result<ExitStatus>) {
    let test_script_path = GLOBAL_TEST_SCRIPT_PATH
        .get()
        .expect("We are missing a GLOBAL_TEST_SCRIPT_PATH?!");

    let expected_output: &str = GLOBAL_EXPECTED_RESULT
        .get()
        .map(|s| s.as_str())
        .unwrap_or("");

    // Build an owned `Command` here:
    let mut binding = Command::new(test_script_path);

    let cmd = binding.arg(query).arg(expected_output);

    // Return it by value (not by reference):
    (cmd.output(), cmd.status())
}
