mod delta_debug;
mod driver;
mod parser;
mod reducer;
mod transformation;
mod utils;
pub mod statements;

use crate::delta_debug::delta_debug;
use log::*;
use std::time::Instant;

// ./reducer –query <query-to-minimize –test <an arbitrary-script>
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now(); // start timing

    let (args, pwd) = utils::init();

    let (query, test_path, query_path) = utils::read_and_parse_args(args, pwd);
    driver::init_query(&query, test_path)?;
    info!("Starting the parser");

    let parsini: Vec<String> = query
        .replace(";;", ";")
        .replace('\n', " ")
        .replace('\r', "")
        .trim()
        .split(';')
        .filter(|part| !part.to_string().is_empty())
        .map(|part| part.to_string())
        .collect();

    info!("parsed query with params: {:?}", parsini.len());
    info!("starting reduction");
    let reduced = delta_debug(parsini.clone(), 2)?;
    info!("query reduced with params {:?}", reduced.len());

    /*&let ast = parser::generate_ast(&query)
            .and_then(reducer::reduce)
            .and_then(|ast| vec_statement_to_string(&ast, "\n"));
    */
    info!("writing results to file");
    utils::print_result(&query_path, &query, &reduced, start.elapsed())
        .expect("TODO: panic message");
    info!("finished writing results to file");
    Ok(())
}
