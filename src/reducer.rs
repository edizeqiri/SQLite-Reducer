use crate::delta_debug::ddmin;
use crate::driver::Setup;
use log::info;
use sqlparser::ast::Expr::UnaryOp;
use sqlparser::ast::Statement;
use sqlparser::*;
use std::io::Error;

pub fn reduce(ast: Vec<Statement>) -> Result<String, Error> {
    info!("{:?}", ast[0].to_string());

    Ok(ast[0].to_string())
}

pub fn reduce_statements(current_ast: Vec<Statement>, setup: Setup) {
    let min_stmt = ddmin(current_ast, setup);
    info!("minimal statement");
    for i in min_stmt {
        info!("{}", i.to_string());
    }
}
