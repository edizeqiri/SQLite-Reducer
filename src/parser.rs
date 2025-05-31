use log::info;
use sqlparser::ast::Statement;
use sqlparser::dialect::SQLiteDialect;
use sqlparser::parser::{Parser, ParserError};
use sqlparser::{ast, dialect};

pub fn generate_ast(sql: &str) -> Result<Vec<Statement>, Box<dyn std::error::Error>> {
    let dialect = dialect::SQLiteDialect {};
    let stmts = Parser::parse_sql(&dialect, sql)?;

    //info!("AST: {:#?}", stmts);
    Ok(stmts)
}
