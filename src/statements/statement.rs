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
}

impl Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Always just print the original SQL text, regardless of variant.
        write!(f, "{}", self.original)
    }
}

pub fn parse_create_table_statements(inputs: Vec<&str>) -> Vec<Statement> {
    // A regex to capture:
    //  1) “CREATE TABLE” (case-insensitive)
    //  2) an optional “IF NOT EXISTS”
    //  3) the table name (anything up to the next ‘(’)
    //  4) everything inside (…) as one group of “columns text”
    //
    // Pattern breakdown (with (?i) for case-insensitive):
    //   (?i)CREATE\s+TABLE        → “create table” in any case, with ≥1 space
    //   (?:\s+IF\s+NOT\s+EXISTS)? → optional “ IF NOT EXISTS”
    //   \s+([^(]+)                → capture group #1: table name (all chars up to ‘(’)
    //   \s*\(\s*([^)]*)\s*\)      → capture group #2: contents of “( … )” (columns block)
    //
    let re = Regex::new(
        r"(?i)^\s*CREATE\s+TABLE(?:\s+IF\s+NOT\s+EXISTS)?\s+([^(]+)\s*\(\s*([^)]*)\s*\)",
    )
    .expect("Regex should compile");

    let mut results = Vec::new();

    for raw in inputs {
        // Try to match. If it fails, skip this string (or you could push an Unknown).
        if let Some(caps) = re.captures(raw) {
            // Capture group 1 is the table name (with possible trailing spaces)
            let table_name = caps[1].trim().to_string();
            // Capture group 2 is the comma-separated “col_name  col_type, col2  type2, …”
            let cols_block = caps[2].trim();

            let mut columns_map = HashMap::new();

            // Split on commas. Then for each piece, split off the first whitespace →
            // left is column name, right is type (the remainder of that piece).
            for col_def in cols_block.split(',') {
                let col_def = col_def.trim();
                if col_def.is_empty() {
                    continue;
                }
                // We look for the first run of whitespace to separate name vs. type
                // e.g. “table_0_c0 TEXT” → name = “table_0_c0”, typ = “TEXT”
                if let Some(idx) = col_def.find(char::is_whitespace) {
                    let name = col_def[..idx].to_string();
                    let col_type = col_def[idx..].trim().to_string();
                    columns_map.insert(name, col_type);
                } else {
                    // If there is no whitespace, we treat entire chunk as a column name
                    // with an empty type. (Adjust this behavior if you expect a guaranteed type.)
                    columns_map.insert(col_def.to_string(), String::new());
                }
            }

            // Build a CreateTable statement:
            let stmt = Statement::new_create_table(raw.to_string(), table_name, columns_map);
            results.push(stmt);
        }
        // else: if it doesn’t match, we simply skip. You could instead push a
        // Statement::Unknown { original: raw.into() } if you prefer.
    }

    results
}

pub fn parse_insert_statements(inputs: Vec<&str>) -> Vec<Statement> {
    // Regex (with (?i) for case-insensitive):
    //  ^\s*INSERT
    //    (?:\s+OR\s+(?:REPLACE|IGNORE))?     → optional “OR REPLACE” or “OR IGNORE”
    //    \s+INTO\s+
    //    ([^(]+)                             → capture group #1 = table name (up to ‘(’)
    //    \s*\(\s*([^)]*)\s*\)                → capture group #2 = columns block
    //    \s*VALUES\s*\(\s*([^)]*)\s*\)       → capture group #3 = values block
    //
    let re = Regex::new(
        r"(?i)^\s*INSERT(?:\s+OR\s+(?:REPLACE|IGNORE))?\s+INTO\s+([^(]+)\s*\(\s*([^)]*)\s*\)\s*VALUES\s*\(\s*([^)]*)\s*\)",
    )
    .expect("Failed to compile INSERT regex");

    let mut results = Vec::new();

    for raw in inputs {
        if let Some(caps) = re.captures(raw) {
            // 1) Extract and trim the table name
            let table_name = caps[1].trim().to_string();
            // 2) Extract the comma-separated list of column names
            let cols_block = caps[2].trim();
            // 3) Extract the comma-separated list of values (as raw strings, including quotes/NULL/TRUE/FALSE/etc.)
            let vals_block = caps[3].trim();

            // Split out column names:
            let col_names: Vec<String> = cols_block
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            // Split out values (we assume no nested commas inside a single literal for these examples):
            let col_values: Vec<String> = vals_block
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            // Zip them into a HashMap<String,String>. If the lengths differ,
            // we pair up only to min(len(names), len(values)).
            let mut columns_and_values = HashMap::new();
            for (name, value) in col_names.iter().zip(col_values.iter()) {
                columns_and_values.insert(name.clone(), value.clone());
            }

            // Build a Statement::Insert
            let stmt = Statement::new_insert(raw.to_string(), table_name, columns_and_values);
            results.push(stmt);
        }
        // else: skip any line that does not match our INSERT pattern
    }

    results
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

    let parsed = parse_create_table_statements(inputs);

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

    let parsed = parse_insert_statements(inputs);

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
