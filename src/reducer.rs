use std::io::empty;
use std::vec;

use crate::bruteforce_debug::{self, bruteforce_delta_debug};
use crate::delta_debug::{self, delta_debug};
use crate::delta_debug_stmt::delta_debug_stmt;
use crate::parser::{generate_ast, sqlparser_generate_ast};
use crate::statements::types::{Statement, StatementKind};
use crate::transformation::transformer::transform;
use crate::utils::vec_statement_to_string;
use log::{info, warn};

pub fn reduce(current_ast: Vec<Statement>) -> Result<String, Box<dyn std::error::Error>> {
    let reduced = delta_debug(current_ast, 2)?;
    //let reduced = remove_table(&reduced)?;

    let query = vec_statement_to_string(&reduced, ";")?;

    let reduced = match sqlparser_generate_ast(&query) {
        Ok(ast) => match vec_statement_to_string(&transform(ast), ";") {
            Ok(result) => result,
            Err(e) => {
                warn!("Failed to transform query: {}. Using original query.", e);
                query
            }
        },
        Err(e) => query,
    };

    // get env QUICK_RUN, if it is true then dont run this
    if std::env::var("QUICK_RUN").is_ok() {
        return Ok(reduced);
    }
    let reduced = brute_force(&reduced)?;

    Ok(reduced)
}

fn brute_force(queries: &String) -> Result<String, Box<dyn std::error::Error>> {
    let mut vec_query: Vec<String> = queries.split(';').map(str::to_string).collect();

    for (index, stmt) in queries.split(";").into_iter().enumerate() {
        let query_vec: Vec<String> = stmt
            .replace("(", " ( ")
            .replace(")", " ) ")
            .split(' ')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .collect();

        let reduced = bruteforce_delta_debug(query_vec, 2, index, &vec_query)?;
        vec_query[index] = reduced;
    }

    Ok(vec_query.join(";"))
}

fn remove_table(queries: &Vec<Statement>) -> Result<Vec<Statement>, Box<dyn std::error::Error>> {
    let table_names: Vec<&String> = queries
        .iter()
        .filter_map(|stmt| stmt.get_create_table_name())
        .collect();

    let all_tables = delta_debug_stmt(table_names, 2, queries);
    info!("removed these tables: {:?}", all_tables);

    Ok(remove_tables_in_place(&all_tables?, queries))
}

pub fn remove_table_in_place(table: &str, queries: Vec<Statement>) -> Vec<Statement> {
    // First remove CREATE TABLE and INSERT statements for the table
    let mut filtered_queries: Vec<Statement> = queries
        .into_iter()
        .filter(|stmt| {
            !matches!(
                &stmt.kind,
                StatementKind::CreateTable { name, .. } if name == table
            ) && !matches!(
                &stmt.kind,
                StatementKind::Insert { table: tbl, .. } if tbl == table
            ) && !matches!(
                &stmt.kind,
                StatementKind::CreateView { name, .. } if name == table
            ) && !matches!(
                &stmt.kind,
                StatementKind::Update { table: tbl, .. } if tbl == table
            ) && !matches!(
                &stmt.kind,
                StatementKind::Delete { table: tbl, .. } if tbl == table
            ) && !matches!(
                &stmt.kind,
                StatementKind::AlterTable { table: tbl, .. } if tbl == table
            )
        })
        .collect();

    for stmt in &mut filtered_queries {
        stmt.remove_table_references(table);
    }

    filtered_queries.retain(|stmt| !stmt.original.is_empty());

    filtered_queries
}

pub fn remove_tables_in_place<T: AsRef<str>>(
    tables: &[T],
    queries: &Vec<Statement>,
) -> Vec<Statement> {
    tables
        .iter()
        .fold(queries.clone(), |current_queries, table| {
            remove_table_in_place(table.as_ref(), current_queries)
        })
}

#[test]
fn test_remove_query2() {
    let query = "CREATE TABLE IF NOT EXISTS t_DX44 (c_LGUf NUMERIC, c_Hlmf3w REAL DEFAULT 749171.692897985, c_ewZ TEXT, c_EwP TEXT DEFAULT 'Fn58MvfLqzQ2DMC4', c_YBA7sBV TEXT CHECK (length(c_YBA7sBV) > 0));
    INSERT OR FAIL INTO t_DX44 (c_LGUf, c_Hlmf3w, c_ewZ, c_EwP, c_YBA7sBV) VALUES (-958347, 803354.0705377955, 'MQ_2', 'qrZM84MTMHUkkov_3', 'IcJ_4'), (1119541, 661160.0780749931, '7131k8CH2I7rflmaZmFh_102', '1sGjUivjzF_103', 'fwAI_104'), (2703615, 419682.84648422664, '6u2sAbJVjXHWP_202', 'YpYYmjS_203', 'AyMTHlf_204');
    SELECT EXISTS (SELECT 1 FROM t_DX44 LIMIT 1) AS alias_xvE FROM t_DX44 WHERE NOT (t_DX44.c_EwP / t_DX44.c_ewZ) GROUP BY c_ewZ, c_Hlmf3w, c_LGUf HAVING CASE WHEN REPLACE(t_DX44.c_YBA7sBV, '7ZjVE', -109744) THEN t_DX44.c_LGUf ELSE TRUE END ORDER BY c_LGUf DESC, c_YBA7sBV;";

    let ast = generate_ast(query).unwrap();
    println!("{:#?}", ast);
    let cleaned = remove_table_in_place("t_DX44", ast);

    print!("{:#?}", cleaned);
    println!("{:#?}", vec_statement_to_string(&cleaned, ";"));

    assert_eq!(1, cleaned.len())
}

#[test]
fn test_remove() {
    let query = "CREATE TABLE  table_0 (table_0_c0 TEXT, table_0_c1 REAL ) ;
        CREATE TABLE IF NOT EXISTS table_1  (table_1_c0 REAL ) ;
        CREATE TABLE  table_2  (table_2_c0 UNSIGNED BIG INT, table_2_c1 BIGINT, table_2_c2 BIGINT ) ;
        CREATE TABLE IF NOT EXISTS table_3 (table_3_c0 UNSIGNED BIG INT, table_3_c1 DATETIME ) ;
        CREATE TABLE  table_4  (table_4_c0 INT, table_4_c1 BOOLEAN, table_4_c2 INT ) ;
        WITH cte_3 AS ( SELECT  * FROM table_1 ) SELECT DISTINCT table_1_c0 FROM table_0, table_1 JOIN table_3 ON table_0.table_0_c1 < table_3.table_3_c0 WHERE EXISTS ( SELECT  * FROM table_3 ORDER BY table_3_c0 LIMIT 1 ) GROUP BY table_3_c0 ORDER BY table_0_c0 ASC LIMIT 0;
        ;;
        ALTER TABLE table_1 ADD alter_table_1_c0 DATETIME ;;
        PRAGMA synchronous ;;
        DELETE FROM table_1 WHERE LOWER ( 1 ) ;;
        SELECT  AVG(table_1_c0) FROM table_1, table_0, table_2 WHERE 1 IS NULL GROUP BY table_2_c1 HAVING IFNULL ( 1 , 1 ) LIMIT 2 OFFSET 2;
        ;;
        ANALYZE table_4 ;;
        ;;
        CREATE TRIGGER trigger_5 BEFORE INSERT ON table_0 BEGIN DELETE FROM table_2 ;
        UPDATE table_1 SET table_1_c0 = 0.0 WHERE IFNULL ( 1 , 1 ) ; END;";

    let ast = generate_ast(query).unwrap();
    let cleaned = remove_table_in_place("table_1", ast);

    println!("{:#?}", cleaned);
    println!("{:#?}", vec_statement_to_string(&cleaned, ";").unwrap());

    assert_eq!(10, cleaned.len())
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
    let cleaned = remove_table_in_place("table_1", ast);
    println!("{:#?}", vec_statement_to_string(&cleaned, ";").unwrap());
    println!("{:?}", cleaned.len());

    assert_eq!(2, cleaned.len())
}
