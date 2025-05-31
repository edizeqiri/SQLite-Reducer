mod delta_debug;
mod driver;
mod parser;
mod reducer;
mod transformation;
mod utils;

use crate::utils::vec_statement_to_string;
use log::*;
use crate::delta_debug::delta_debug;

// ./reducer –query <query-to-minimize –test <an arbitrary-script>
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (args, pwd) = utils::init();

    let (query, test_path) = utils::read_and_parse_args(args, pwd);
    driver::init_query(&query, test_path)?;
    
    let parsini = &query.replace(";;",";").replace("\n"," ").split(";").map(|x| x.to_string()).collect::<Vec<String>>();
    
    delta_debug(parsini.clone(), 2)?;
    
    
    

    /*&let ast = parser::generate_ast(&query)
        .and_then(reducer::reduce)
        .and_then(|ast| vec_statement_to_string(&ast, "\n"));
*/
    warn!("[ANALYSIS] ast: {:?}", parsini.join(";"));
    Ok(())
}
