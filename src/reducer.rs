use std::io::Error;
use log::info;
use sqlparser::ast::Statement;
use sqlparser::*;
use sqlparser::ast::Expr::UnaryOp;

pub fn reduce(ast: Vec<Statement>) -> Result<String,Error> {
    info!("{:?}", ast[0].to_string());

    Ok(ast[0].to_string())
}
