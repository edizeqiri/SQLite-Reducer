use crate::statements::parsers::{parse_create_table_statement, parse_insert_statement};
use crate::statements::types::StatementKind;

#[test]
fn test_create_q3() {
    let inputs = vec![
        "CREATE TABLE  table_0 (table_0_c0 TEXT, table_0_c1 REAL ) ;",
        "CREATE TABLE IF NOT EXISTS table_1  (table_1_c0 REAL ) ;",
        "CREATE TABLE  table_2  (table_2_c0 UNSIGNED BIG INT, table_2_c1 BIGINT, table_2_c2 BIGINT ) ;",
        "CREATE TABLE IF NOT EXISTS table_3 (table_3_c0 UNSIGNED BIG INT, table_3_c1 DATETIME ) ;",
        "CREATE TABLE  table_4  (table_4_c0 INT, table_4_c1 BOOLEAN, table_4_c2 INT ) ;",
    ];

    let parsed: Vec<_> = inputs
        .iter()
        .map(|s| parse_create_table_statement(s).expect("should parse CREATE TABLE"))
        .collect();

    for stmt in parsed {
        match stmt.kind {
            StatementKind::CreateTable {
                ref name,
                ref columns,
            } => {
                println!("Table: {}", name);
                for (col, ty) in columns {
                    println!("  · {}  → {}", col, ty);
                }
                println!("  (original SQL: {})\n", stmt);
            }
            _ => (),
        }
    }
}

#[test]
fn insert_test_q3() {
    let inputs = vec![
        "INSERT INTO table_0 (table_0_c0, table_0_c1) VALUES ('bob', -0.0) ;",
        "INSERT INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (FALSE, TRUE, 4) ;",
        "INSERT OR REPLACE INTO table_1 (table_1_c0) VALUES (2.0) ;",
        "INSERT INTO table_0 (table_0_c0, table_0_c1) VALUES (NULL, 2.0) ;",
        "INSERT OR IGNORE INTO table_0 (table_0_c0, table_0_c1) VALUES ('switzerland', 1.5) ;",
        "INSERT INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (1, FALSE, -2) ;",
        "INSERT INTO table_2 (table_2_c0, table_2_c1, table_2_c2) VALUES (FALSE, -1, 1) ;",
        "INSERT OR REPLACE INTO table_3 (table_3_c0, table_3_c1) VALUES (1, NULL) ;",
        "INSERT OR REPLACE INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (NULL, NULL, 4) ;",
        "INSERT INTO table_2 (table_2_c0, table_2_c1, table_2_c2) VALUES (-2, 2, -0) ;",
        "INSERT INTO table_3 (table_3_c0, table_3_c1) VALUES (-1, NULL) ;",
        "INSERT OR IGNORE INTO table_0 (table_0_c0, table_0_c1) VALUES ('alice', -1.5) ;",
    ];

    let parsed: Vec<_> = inputs
        .iter()
        .map(|s| parse_insert_statement(s).expect("should parse INSERT"))
        .collect();

    for stmt in parsed {
        if let StatementKind::Insert {
            ref table,
            ref columns_and_values,
        } = stmt.kind
        {
            println!("Table: {}", table);
            for (col, val) in columns_and_values {
                println!("  · {} → {}", col, val);
            }
            println!("  (original SQL: {})\n", stmt);
        }
    }
}
