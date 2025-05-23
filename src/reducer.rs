use log::info;
use sqlparser::ast::Expr::UnaryOp;
use sqlparser::ast::Statement;
use sqlparser::*;
use std::io::Error;

pub fn reduce(ast: Vec<Statement>) -> Result<String, Error> {
    info!("{:?}", ast[0].to_string());

    Ok(ast[0].to_string())
}
