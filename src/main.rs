mod delta_debug;
mod driver;
mod parser;
mod reducer;
mod transformation;
mod utils;

use std::time::{Duration, Instant};
use env_logger::init_from_env;
use crate::delta_debug::delta_debug;
use crate::utils::vec_statement_to_string;
use log::*;

// ./reducer –query <query-to-minimize –test <an arbitrary-script>
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now(); // start timing
    
    let (args, pwd) = utils::init();

    let (query, test_path) = utils::read_and_parse_args(args, pwd);
    driver::init_query(&query, test_path)?;
    info!("query {:?}", &query);

    let parsini = &query
        .replace(";;", ";")
        .replace("\n", " ")
        .replace("\r", "")
        .split(";")
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    let reduced = delta_debug(parsini.clone(), 2)?;
    let reduced_query = reduced.join(";") + ";";

    /*&let ast = parser::generate_ast(&query)
            .and_then(reducer::reduce)
            .and_then(|ast| vec_statement_to_string(&ast, "\n"));
    */
    
    utils::print_result(&query, &reduced_query, start.elapsed()).expect("TODO: panic message");
    Ok(())
}


