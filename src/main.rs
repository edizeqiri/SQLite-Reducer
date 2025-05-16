use clap::Parser;
use std::path::PathBuf;
use log::*;
use std::{env, fs};
use std::path::Component::CurDir;
use std::process::Command;

// ./reducer –query <query-to-minimize –test <an arbitrary-script>
fn main() {
    let (args, pwd) = init();

    read_and_parse_args(args,pwd);

}

fn testQuery(test: String, reducedQueryFile: String) {

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "echo hello"])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("echo hello")
            .output()
            .expect("failed to execute process")
    };

    let hello = output.stdout;
}

fn read_and_parse_args(args: Cli, pwd: PathBuf) {
    let query = fs::read_to_string(pwd.join(args.query))
        .expect("Should have been able to read the query file");

    let test = fs::read_to_string(pwd.join(args.test))
        .expect("Should have been able to read the test file");
    info!("Query is:\n{query}");
}

fn init() -> (Cli, PathBuf) {
    env_logger::init();
    let args = Cli::parse();

    info!("query: {:?}, test: {:?}", args.query, args.test);
    let pwd: PathBuf = env::current_dir().unwrap();
    println!("Current directory: {}", pwd.display());
    (args, pwd)
}

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    query: PathBuf,
    #[arg(long)]
    test: PathBuf,
}
