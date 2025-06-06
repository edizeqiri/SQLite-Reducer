use crate::delta_debug::delta_debug;
use crate::transformation::transformer::transform;
use log::info;
use sqlparser::ast::Statement;

pub fn reduce(current_ast: Vec<Statement>) -> Result<Vec<Statement>, Box<dyn std::error::Error>> {
    let current_ast_length = current_ast.len();

    let minimal_stmt = delta_debug(current_ast, 2).map(transform);

    info!(
        "original query length: {:?}, reduced query length: {:?}",
        current_ast_length,
        minimal_stmt.as_ref().map(|v| v.len())
    );

    minimal_stmt
}
