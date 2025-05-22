use log::info;
use sqlparser::ast::Statement;

pub fn reduce(ast: Vec<Statement>) {
    info!("{:?}", ast.first().unwrap());
}
