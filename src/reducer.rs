use crate::delta_debug::delta_debug;
use crate::parser::generate_ast;
use crate::statements::types::{Statement, StatementKind};
use crate::utils::vec_statement_to_string;
use log::info;

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

fn table_insert_play(
    queries: Vec<Statement>,
) -> Result<Vec<Statement>, Box<dyn std::error::Error>> {
    let table_names: Vec<&String> = queries
        .iter()
        .filter_map(|stmt| stmt.get_create_table_name())
        .collect();

    todo!()
}

pub fn remove_table_in_place(table: &str, mut queries: Vec<Statement>) -> Vec<Statement> {
    // First remove CREATE TABLE and INSERT statements for the table
    queries.retain(|stmt| {
        !matches!(
            &stmt.kind,
            StatementKind::CreateTable { name, .. } if name == table
        ) && !matches!(
            &stmt.kind,
            StatementKind::Insert { table: tbl, .. } if tbl == table
        ) && !matches!(
            &stmt.kind,
            StatementKind::CreateView { name, .. } if name == table
        )
    });

    // Then remove table references from remaining statements
    for stmt in &mut queries {
        stmt.remove_table_references(table);
    }

    queries
}

#[test]
fn test_remove() {
    let query = "CREATE TABLE  table_0 (table_0_c0 TEXT, table_0_c1 REAL ) ;
        CREATE TABLE IF NOT EXISTS table_1  (table_1_c0 REAL ) ;
        CREATE TABLE  table_2  (table_2_c0 UNSIGNED BIG INT, table_2_c1 BIGINT, table_2_c2 BIGINT ) ;
        CREATE TABLE IF NOT EXISTS table_3 (table_3_c0 UNSIGNED BIG INT, table_3_c1 DATETIME ) ;
        CREATE TABLE  table_4  (table_4_c0 INT, table_4_c1 BOOLEAN, table_4_c2 INT ) ;
        INSERT INTO table_3 (table_3_c0, table_3_c1) VALUES (-2, NULL) ;
        INSERT OR IGNORE INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (3, 0, TRUE) ;
        INSERT INTO table_0 (table_0_c0, table_0_c1) VALUES ('male', 1.5) ;
        INSERT INTO table_3 (table_3_c0, table_3_c1) VALUES (4, NULL) ;
        INSERT INTO table_2 (table_2_c0, table_2_c1, table_2_c2) VALUES (1, -0, 0) ;
        INSERT INTO table_3 (table_3_c0, table_3_c1) VALUES (2, NULL) ;
        INSERT INTO table_1 (table_1_c0) VALUES (-1.5) ;
        INSERT OR REPLACE INTO table_2 (table_2_c0, table_2_c1, table_2_c2) VALUES (3, 3, -1) ;
        INSERT INTO table_1 (table_1_c0) VALUES (-0.0) ;
        INSERT INTO table_3 (table_3_c0, table_3_c1) VALUES (TRUE, NULL) ;
        INSERT OR IGNORE INTO table_3 (table_3_c0, table_3_c1) VALUES (3, NULL) ;";

    let ast = generate_ast(query).unwrap();
    let cleaned = remove_table_in_place("table_0", ast);

    print!("{:?}", cleaned);

    assert_eq!(14, cleaned.len())
}

#[test]
fn test_select_remover() {
    let query = "CREATE TABLE  table_0 (table_0_c0 TEXT, table_0_c1 REAL ) ;
        CREATE TABLE IF NOT EXISTS table_1  (table_1_c0 REAL ) ;
        CREATE VIEW view_0 AS WITH cte_1 AS ( SELECT DISTINCT * FROM table_0, table_3, table_2 ) SELECT DISTINCT * FROM table_1, table_4, table_0 ;;
        ";
    let result = "CREATE TABLE IF NOT EXISTS table_1  (table_1_c0 REAL );
        CREATE VIEW view_0 AS WITH cte_1 AS ( SELECT DISTINCT * FROM  table_3, table_2 ) SELECT DISTINCT * FROM table_1, table_4;";

    let ast = generate_ast(query).unwrap();
    println!("{:#?}", ast);
    let cleaned = remove_table_in_place("table_0", ast);
    println!("{:#?}", vec_statement_to_string(&cleaned, ";").unwrap());
    println!("{:?}", cleaned.len());

    assert_eq!(2, cleaned.len())
}
