use crate::delta_debug::delta_debug;
use log::info;
use crate::statements::statement::Statement;

pub fn reduce(current_ast: Vec<Statement>) -> Result<Vec<Statement>, Box<dyn std::error::Error>> {
    let current_ast_length = current_ast.len();

    let minimal_stmt = delta_debug(current_ast, 2)?;

    info!(
        "original query length: {:?}, reduced query length: {:?}",
        current_ast_length,
        minimal_stmt.len()
    );

    Ok(minimal_stmt)
}

/* fn table_insert_play(queries: Vec<Statement>) -> Result<Vec<Statement>, Box<dyn std::error::Error>> {

    queries.iter().map(|query|  )
    Ok()
} */