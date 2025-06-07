use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use regex::Regex;


#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    original: String,
    kind: StatementKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    CreateTable { name: String, columns: HashMap<String, String> },
    Insert      { table: String, columns_and_values: HashMap<String, String> },
    Select      { /* if you had select‐specific fields, they'd go here */ },
    CreateView  { /* if you had view‐specific fields, they'd go here */ },
    Drop        { /* …drop‐specific fields… */ },
    Unknown     { /* … */ },
}

impl Statement {
    /// Construct a CREATE TABLE statement.
    pub fn new_create_table(original: String, name: String, columns: HashMap<String, String>) -> Self {
        Statement {
            original,
            kind: StatementKind::CreateTable { name, columns },
        }
    }

    /// Construct an INSERT statement.
    pub fn new_insert(original: String, table: String, columns_and_values: HashMap<String, String>) -> Self {
        Statement {
            original,
            kind: StatementKind::Insert { table, columns_and_values },
        }
    }

    pub fn new(original: &str) -> Self {
        Statement {
            original: original.to_string(),
            kind: StatementKind::Unknown {  }
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Always just print the original SQL text, regardless of variant.
        write!(f, "{}", self.original)
    }
}


pub fn parse_insert_statement(query: &str) -> Result<Statement, Box<dyn std::error::Error>> {
    // Compile our INSERT regex (case‐insensitive)
    let re = Regex::new(
        r"(?i)^\s*INSERT(?:\s+OR\s+(?:REPLACE|IGNORE))?\s+INTO\s+([^(]+)\s*\(\s*([^)]*)\s*\)\s*VALUES\s*\(\s*([^)]*)\s*\)"
    )?;

    // Try to capture table name, columns‐block, values‐block
    let caps = re
        .captures(query)
        .ok_or_else(|| format!("Not a valid INSERT statement: {}", query))?;

    // 1) Table name
    let table = caps[1].trim().to_string();
    // 2) Columns (comma‐separated names)
    let cols_block = caps[2].trim();
    let col_names: Vec<String> = cols_block
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    // 3) Values (comma‐separated literals)
    let vals_block = caps[3].trim();
    let col_values: Vec<String> = vals_block
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    // Zip into HashMap<column, value>
    let mut columns_and_values = HashMap::new();
    for (name, value) in col_names.iter().zip(col_values.iter()) {
        columns_and_values.insert(name.clone(), value.clone());
    }

    // Build and return our Statement
    Ok(Statement::new_insert(query.to_string(), table, columns_and_values))
}


pub fn parse_create_table_statement(query: &str) -> Result<Statement, Box<dyn std::error::Error>> {
    // Compile the regex (case-insensitive, optional IF NOT EXISTS)
    let re = Regex::new(
        r"(?i)^\s*CREATE\s+TABLE(?:\s+IF\s+NOT\s+EXISTS)?\s+([^(]+)\s*\(\s*([^)]*)\s*\)"
    )?;

    // Try to match and capture:
    // 1 = table name, 2 = contents of ( … ) as a single string
    let caps = re
        .captures(query)
        .ok_or_else(|| format!("Not a valid CREATE TABLE statement: {}", query))?;

    // 1) Extract & trim table name
    let name = caps[1].trim().to_string();
    // 2) Extract & trim columns block
    let cols_block = caps[2].trim();

    // Build the column → type map
    let mut columns = HashMap::new();
    for part in cols_block.split(',') {
        let def = part.trim();
        if def.is_empty() {
            continue;
        }
        if let Some(idx) = def.find(char::is_whitespace) {
            let col = def[..idx].to_string();
            let ty  = def[idx..].trim().to_string();
            columns.insert(col, ty);
        } else {
            // no whitespace? treat the whole thing as column name
            columns.insert(def.to_string(), String::new());
        }
    }

    // Return the parsed statement
    Ok(Statement::new_create_table(query.to_string(), name, columns))
}

#[test]
fn test_create_q3() {
    let inputs = vec![
        "CREATE TABLE  table_0 (table_0_c0 TEXT, table_0_c1 REAL ) ;",
        "CREATE TABLE IF NOT EXISTS table_1  (table_1_c0 REAL ) ;",
        "CREATE TABLE  table_2  (table_2_c0 UNSIGNED BIG INT, table_2_c1 BIGINT, table_2_c2 BIGINT ) ;",
        "CREATE TABLE IF NOT EXISTS table_3 (table_3_c0 UNSIGNED BIG INT, table_3_c1 DATETIME ) ;",
        "CREATE TABLE  table_4  (table_4_c0 INT, table_4_c1 BOOLEAN, table_4_c2 INT ) ;",
    ];

    let parsed: Vec<Statement> = inputs
            .iter()
            .map(|s| parse_create_table_statement(s).expect("should parse CREATE TABLE"))
            .collect();

    for stmt in parsed {
        match stmt.kind {
            StatementKind::CreateTable { ref name, ref columns } => {
                println!("Table: {}", name);
                for (col, ty) in columns {
                    println!("  · {}  → {}", col, ty);
                }
                // Display itself prints the original SQL:
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

    let parsed: Vec<Statement> = inputs
            .iter()
            .map(|s| parse_insert_statement(s).expect("should parse CREATE TABLE"))
            .collect();

    for stmt in parsed {
        if let StatementKind::Insert { ref table, ref columns_and_values } = stmt.kind {
            println!("Table: {}", table);
            for (col, val) in columns_and_values {
                println!("  · {} → {}", col, val);
            }
            println!("  (original SQL: {})\n", stmt);
        }
    }
}
