use log::info;
use sqlparser::ast::Statement;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::{Parser, ParserError};

pub fn generate_ast(sql: &str) -> Result<Vec<Statement>, ParserError> {
    let dialect = GenericDialect {};
    let stmts = Parser::parse_sql(&dialect, sql)?;
    info!("AST: {:#?}", stmts);
    Ok(stmts)
}
