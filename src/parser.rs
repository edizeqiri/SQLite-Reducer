use log::info;
use sqlparser::ast::Statement;
use sqlparser::dialect::SQLiteDialect;
use sqlparser::parser::{Parser, ParserError};

pub fn generate_ast(sql: &str) -> Result<Vec<Statement>, ParserError> {
    let dialect = SQLiteDialect {};
    let stmts = Parser::parse_sql(&dialect, sql)?;
    //info!("AST: {:#?}", stmts);
    Ok(stmts)
}

#[test]
fn test_to_string() {
    let query = "SELECT 2 + 3 * (4 - 1);INSERT 1 INTO F;";
    let ast = generate_ast(query);
    println!("{:?}", ast.unwrap().first().unwrap().to_string());
}