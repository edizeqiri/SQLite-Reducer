use log::info;
use sqlparser::ast::Statement;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

pub fn generate_ast(sql: &str) -> Vec<Statement> {
    let dialect = GenericDialect {}; // or AnsiDialect, or your own dialect ...
    let ast = Parser::parse_sql(&dialect, sql);
    info!("AST: {:?}", ast);
    ast.unwrap_or_default()
}


