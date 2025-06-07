mod delta_debug;
mod driver;
mod parser;
mod reducer;
pub mod statements;
mod transformation;
mod utils;

use crate::{parser::generate_ast, reducer::reduce, utils::vec_statement_to_string};
use log::*;
use std::time::Instant;

// ./reducer –query <query-to-minimize –test <an arbitrary-script>
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now(); // start timing

    let (args, pwd) = utils::init();

    let (query, test_path, query_path) = utils::read_and_parse_args(args, pwd);
    driver::init_query(&query, test_path)?;
    info!("Starting the parser");

    let parsed_query = generate_ast(&query)?;

    info!("parsed query with params: {:?}", parsed_query.len());
    info!("starting reduction");
    let reduced = reduce(parsed_query)?;
    info!("query reduced with params {:?}", reduced.len());

    info!("{:?}", vec_statement_to_string(&reduced, "\n"));

    info!("writing results to file");
    utils::print_result(
        &query_path,
        &query,
        &vec_statement_to_string(&reduced, ";")?,
        start.elapsed(),
    )
    .expect("TODO: panic message");
    info!("finished writing results to file");
    Ok(())
}
