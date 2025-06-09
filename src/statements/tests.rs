use crate::statements::parsers::{
    parse_create_table, parse_insert_statement, parse_select_statement,
};
use crate::statements::types::{Column, StatementKind};

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
        .map(|s| parse_create_table(s).expect("should parse CREATE TABLE"))
        .collect();

    for stmt in parsed {
        match stmt.kind {
            StatementKind::CreateTable {
                ref name,
                ref columns,
            } => {
                println!("Table: {}", name);
                for col in columns {
                    println!("  · {}  → {}", col.name, col.table.as_deref().unwrap_or(""));
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
            ref columns,
            ref values,
        } = stmt.kind
        {
            println!("Table: {}", table);
            for (col, val) in columns.iter().zip(values[0].iter()) {
                println!("  · {} → {}", col, val);
            }
            println!("  (original SQL: {})\n", stmt);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::statements::parsers::{
        parse_create_table, parse_insert_statement, parse_select_statement,
    };
    use crate::statements::types::{Column, StatementKind};

    #[test]
    fn test_create_table_parsing() {
        let sql = "CREATE TABLE test_table (col1 TEXT, col2 INTEGER)";
        let stmt = parse_create_table(sql).unwrap();
        if let StatementKind::CreateTable { name, columns } = stmt.kind {
            assert_eq!(name, "test_table");
            assert_eq!(columns.len(), 2);
            assert_eq!(columns[0].name, "col1");
            assert_eq!(columns[1].name, "col2");
        } else {
            panic!("Expected CreateTable variant");
        }
    }

    #[test]
    fn test_insert_parsing() {
        let sql = "INSERT INTO test_table (col1, col2) VALUES ('value1', 42), ('value2', 43)";
        let stmt = parse_insert_statement(sql).unwrap();
        if let StatementKind::Insert {
            table,
            columns,
            values,
        } = stmt.kind
        {
            assert_eq!(table, "test_table");
            assert_eq!(columns, vec!["col1", "col2"]);
            assert_eq!(values.len(), 2);
            assert_eq!(values[0], vec!["'value1'", "42"]);
            assert_eq!(values[1], vec!["'value2'", "43"]);
        } else {
            panic!("Expected Insert variant");
        }
    }

    #[test]
    fn test_select_parsing() {
        let sql = "SELECT col1, col2 FROM test_table WHERE col1 = 'value'";
        let stmt = parse_select_statement(sql).unwrap();
        if let StatementKind::Select {
            columns,
            tables,
            conditions,
            ..
        } = stmt.kind
        {
            assert_eq!(columns.len(), 2);
            assert_eq!(tables, vec!["test_table"]);
            assert_eq!(conditions.len(), 1);
        } else {
            panic!("Expected Select variant");
        }
    }
}
