use crate::delta_debug::ddmin;
use crate::driver::Setup;
use crate::transformation;
use log::info;
use sqlparser::ast::Statement;
use std::io::Error;
use transformation::transformer;

pub fn reduce(ast: Vec<Statement>) -> Result<String, Error> {
    //info!("{:?}", ast[0].to_string());
    let trans = transformer::transform(ast.clone());
    info!("Transformation is : {:?}", trans);
    Ok("Print".to_string())
}

pub fn reduce_statements(current_ast: Vec<Statement>, setup: Setup) {
    let min_stmt = ddmin(current_ast, setup);
    info!("minimal statement");
    for i in min_stmt {
        info!("{}", i.to_string());
    }
}
