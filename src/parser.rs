use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;


use crate::statements::{parsers, types::Statement};


pub fn sqlparser_generate_ast(sql: &str) -> Result<Vec<sqlparser::ast::Statement>, Box<dyn std::error::Error>> {
    let dialect = GenericDialect::default();
    let ast = Parser::parse_sql(&dialect, sql)?;
    Ok(ast)
}

pub fn generate_ast(sql: &str) -> Result<Vec<Statement>, Box<dyn std::error::Error>> {
    let mut parsed_queries = Vec::new();
    let parsini: Vec<String> = sql
        .replace(";;", ";")
        .replace('\n', " ")
        .replace('\r', "")
        .split(';')
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect();

    for raw in parsini {
        if raw.is_empty() {
            continue;
        }

        // Try each parser in sequence, but handle errors properly
        let stmt = if raw.to_uppercase().starts_with("UPDATE") {
            // For UPDATE statements, try the update parser first
            match parsers::parse_update_statement(&raw) {
                Ok(stmt) => stmt,
                Err(_) => Statement::new(&raw),
            }
        } else if raw.to_uppercase().starts_with("DELETE") {
            // For DELETE statements, try the delete parser first
            match parsers::parse_delete_statement(&raw) {
                Ok(stmt) => stmt,
                Err(_) => Statement::new(&raw),
            }
        } else if raw.to_uppercase().starts_with("ALTER TABLE") {
            // For ALTER TABLE statements, try the alter table parser first
            match parsers::parse_alter_table_statement(&raw) {
                Ok(stmt) => stmt,
                Err(_) => Statement::new(&raw),
            }
        } else {
            // For other statements, try all parsers in sequence
            parsers::parse_create_table(&raw)
                .or_else(|_| parsers::parse_insert_statement(&raw))
                .or_else(|_| parsers::parse_create_view_statement(&raw))
                .or_else(|_| parsers::parse_select_statement(&raw))
                .or_else(|_| parsers::parse_trigger_statement(&raw))
                .or_else(|_| parsers::parse_update_statement(&raw))
                .or_else(|_| parsers::parse_delete_statement(&raw))
                .or_else(|_| parsers::parse_alter_table_statement(&raw))
                .unwrap_or_else(|_| Statement::new(&raw))
        };

        parsed_queries.push(stmt);
    }

    Ok(parsed_queries)
}

#[test]
fn parseQuery3() {
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
INSERT OR IGNORE INTO table_3 (table_3_c0, table_3_c1) VALUES (3, NULL) ;
INSERT INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (TRUE, FALSE, FALSE) ;
INSERT INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (4, 1, 0) ;
INSERT INTO table_0 (table_0_c0, table_0_c1) VALUES ('bob', 0.0) ;
INSERT INTO table_2 (table_2_c0, table_2_c1, table_2_c2) VALUES (-2, -0, NULL) ;
INSERT INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (1, 0, FALSE) ;
INSERT INTO table_1 (table_1_c0) VALUES (2.0) ;
INSERT INTO table_0 (table_0_c0, table_0_c1) VALUES ('bob', -0.0) ;
INSERT INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (FALSE, TRUE, 4) ;
INSERT OR REPLACE INTO table_1 (table_1_c0) VALUES (2.0) ;
INSERT INTO table_0 (table_0_c0, table_0_c1) VALUES (NULL, 2.0) ;
INSERT OR IGNORE INTO table_0 (table_0_c0, table_0_c1) VALUES ('switzerland', 1.5) ;
INSERT INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (1, FALSE, -2) ;
INSERT INTO table_2 (table_2_c0, table_2_c1, table_2_c2) VALUES (FALSE, -1, 1) ;
INSERT OR REPLACE INTO table_3 (table_3_c0, table_3_c1) VALUES (1, NULL) ;
INSERT OR REPLACE INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (NULL, NULL, 4) ;
INSERT INTO table_2 (table_2_c0, table_2_c1, table_2_c2) VALUES (-2, 2, -0) ;
INSERT INTO table_3 (table_3_c0, table_3_c1) VALUES (-1, NULL) ;
INSERT OR IGNORE INTO table_0 (table_0_c0, table_0_c1) VALUES ('alice', -1.5) ;
INSERT INTO table_0 (table_0_c0, table_0_c1) VALUES ('male', TRUE) ;
INSERT OR IGNORE INTO table_1 (table_1_c0) VALUES (2.0) ;
INSERT INTO table_1 (table_1_c0) VALUES (TRUE) ;
INSERT INTO table_3 (table_3_c0, table_3_c1) VALUES (NULL, NULL) ;
INSERT INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (0, 1, -1) ;
INSERT INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (-2, FALSE, FALSE) ;
INSERT INTO table_1 (table_1_c0) VALUES (NULL) ;
INSERT INTO table_1 (table_1_c0) VALUES (-0.0) ;
INSERT INTO table_0 (table_0_c0, table_0_c1) VALUES ('switzerland', -1.5) ;
INSERT INTO table_0 (table_0_c0, table_0_c1) VALUES ('bob', -0.0) ;
INSERT OR REPLACE INTO table_3 (table_3_c0, table_3_c1) VALUES (3, NULL) ;
INSERT INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (3, TRUE, 2) ;
INSERT INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (2, 0, 4) ;
INSERT INTO table_2 (table_2_c0, table_2_c1, table_2_c2) VALUES (NULL, 3, -1) ;
INSERT INTO table_3 (table_3_c0, table_3_c1) VALUES (2, NULL) ;
INSERT INTO table_2 (table_2_c0, table_2_c1, table_2_c2) VALUES (0, -0, NULL) ;
INSERT OR IGNORE INTO table_0 (table_0_c0, table_0_c1) VALUES ('switzerland', 0.0) ;
INSERT INTO table_2 (table_2_c0, table_2_c1, table_2_c2) VALUES (4, 1, FALSE) ;
INSERT INTO table_2 (table_2_c0, table_2_c1, table_2_c2) VALUES (4, 4, -2) ;
INSERT INTO table_1 (table_1_c0) VALUES (-1.5) ;
INSERT INTO table_3 (table_3_c0, table_3_c1) VALUES (FALSE, NULL) ;
INSERT OR REPLACE INTO table_3 (table_3_c0, table_3_c1) VALUES (-1, NULL) ;
INSERT OR IGNORE INTO table_3 (table_3_c0, table_3_c1) VALUES (0, NULL) ;
INSERT INTO table_2 (table_2_c0, table_2_c1, table_2_c2) VALUES (FALSE, FALSE, 2) ;
INSERT INTO table_1 (table_1_c0) VALUES (-1.5) ;
INSERT INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (-0, 1, 0) ;
INSERT INTO table_1 (table_1_c0) VALUES (2.0) ;
INSERT OR IGNORE INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (TRUE, NULL, NULL) ;
INSERT INTO table_3 (table_3_c0, table_3_c1) VALUES (-1, NULL) ;
INSERT INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (4, NULL, NULL) ;
INSERT INTO table_1 (table_1_c0) VALUES (-1.5) ;
INSERT INTO table_3 (table_3_c0, table_3_c1) VALUES (1, NULL) ;
INSERT INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (TRUE, TRUE, -2) ;
INSERT INTO table_0 (table_0_c0, table_0_c1) VALUES ('germany', -1.5) ;
INSERT OR IGNORE INTO table_3 (table_3_c0, table_3_c1) VALUES (TRUE, NULL) ;
INSERT INTO table_1 (table_1_c0) VALUES (-0.0) ;
INSERT INTO table_1 (table_1_c0) VALUES (2.0) ;
INSERT INTO table_2 (table_2_c0, table_2_c1, table_2_c2) VALUES (3, TRUE, NULL) ;
INSERT OR REPLACE INTO table_2 (table_2_c0, table_2_c1, table_2_c2) VALUES (-2, 1, 0) ;
INSERT INTO table_2 (table_2_c0, table_2_c1, table_2_c2) VALUES (4, -0, 3) ;
INSERT INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (TRUE, 1, NULL) ;
INSERT OR IGNORE INTO table_3 (table_3_c0, table_3_c1) VALUES (4, NULL) ;
INSERT INTO table_2 (table_2_c0, table_2_c1, table_2_c2) VALUES (2, 1, 4) ;
INSERT INTO table_3 (table_3_c0, table_3_c1) VALUES (TRUE, NULL) ;
INSERT OR IGNORE INTO table_4 (table_4_c0, table_4_c1, table_4_c2) VALUES (2, TRUE, 0) ;
INSERT OR IGNORE INTO table_1 (table_1_c0) VALUES (-1.5) ;
REINDEX ;;
CREATE VIEW view_0 AS WITH cte_1 AS ( SELECT DISTINCT * FROM table_0, table_3, table_2 ) SELECT DISTINCT * FROM table_1, table_4, table_0 ;;
;;
;;
WITH cte_2 AS ( SELECT  * FROM table_3, table_1 ) SELECT  * FROM table_4 JOIN table_0 ON table_4.table_4_c2 > table_0.table_0_c0 ORDER BY table_4_c0 LIMIT 1;
REINDEX ;;
REINDEX ;;
ANALYZE ;;
DROP VIEW view_0 ;;
WITH cte_3 AS ( SELECT  * FROM table_1 ) SELECT DISTINCT table_1_c0 FROM table_0, table_1 JOIN table_3 ON table_0.table_0_c1 < table_3.table_3_c0 WHERE EXISTS ( SELECT  * FROM table_3 ORDER BY table_3_c0 LIMIT 1 ) GROUP BY table_3_c0 ORDER BY table_0_c0 ASC LIMIT 0;
;;
ALTER TABLE table_1 ADD alter_table_1_c0 DATETIME ;;
PRAGMA synchronous ;;
DELETE FROM table_1 WHERE LOWER ( 1 ) ;;
SELECT  AVG(table_1_c0) FROM table_1, table_0, table_2 WHERE 1 IS NULL GROUP BY table_2_c1 HAVING IFNULL ( 1 , 1 ) LIMIT 2 OFFSET 2;
;;
ANALYZE table_4 ;;
;;
CREATE TRIGGER trigger_5 BEFORE INSERT ON table_0 BEGIN DELETE FROM table_2 ; UPDATE table_1 SET table_1_c0 = 0.0 WHERE IFNULL ( 1 , 1 ) ; END;
 SELECT DISTINCT * FROM table_3, table_2 WHERE EXISTS ( SELECT  table_3_c1 FROM table_3 LIMIT NULL ) LIMIT 3;
";
    let a = generate_ast(query);
    print!("{:?}", a);
}
