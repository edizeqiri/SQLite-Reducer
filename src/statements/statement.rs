use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    original: String,
    pub kind: StatementKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub table: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub table: String,
    pub column: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WithClause {
    pub name: String,
    pub query: Box<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    CreateTable {
        name: String,
        columns: HashMap<String, String>,
    },
    Insert {
        table: String,
        columns_and_values: HashMap<String, String>,
    },
    Select {
        with_clauses: Vec<WithClause>,
        columns: Vec<Column>,
        tables: Vec<String>,
        conditions: Vec<Condition>,
        subqueries: Vec<Statement>,
        is_distinct: bool,
    },
    CreateView {
        name: String,
        query: Box<Statement>,
    },
    Drop,
    Unknown,
}

impl Statement {
    /// Construct a CREATE TABLE statement.
    pub fn new_create_table(
        original: String,
        name: String,
        columns: HashMap<String, String>,
    ) -> Self {
        Statement {
            original,
            kind: StatementKind::CreateTable { name, columns },
        }
    }

    /// Construct an INSERT statement.
    pub fn new_insert(
        original: String,
        table: String,
        columns_and_values: HashMap<String, String>,
    ) -> Self {
        Statement {
            original,
            kind: StatementKind::Insert {
                table,
                columns_and_values,
            },
        }
    }

    pub fn new_select(
        original: String,
        with_clauses: Vec<WithClause>,
        columns: Vec<Column>,
        tables: Vec<String>,
        conditions: Vec<Condition>,
        subqueries: Vec<Statement>,
        is_distinct: bool,
    ) -> Self {
        Statement {
            original,
            kind: StatementKind::Select {
                with_clauses,
                columns,
                tables,
                conditions,
                subqueries,
                is_distinct,
            },
        }
    }

    pub fn new_create_view(original: String, name: String, query: Statement) -> Self {
        Statement {
            original,
            kind: StatementKind::CreateView {
                name,
                query: Box::new(query),
            },
        }
    }

    pub fn new(original: &str) -> Self {
        Statement {
            original: original.to_string(),
            kind: StatementKind::Unknown,
        }
    }

    pub fn get_create_table_name(&self) -> Option<&String> {
        if let StatementKind::CreateTable { name, .. } = &self.kind {
            Some(name)
        } else {
            None
        }
    }

    pub fn remove_table_references(&mut self, table: &str) {
        match &mut self.kind {
            StatementKind::Select {
                with_clauses,
                columns,
                tables,
                conditions,
                subqueries,
                ..
            } => {
                // Remove table from tables list
                tables.retain(|t| t != table);

                // Remove columns referencing the table
                columns.retain(|col| col.table.as_ref() != Some(&table.to_string()));

                // Remove conditions referencing the table
                conditions.retain(|cond| cond.table != table);

                // Process WITH clauses
                for with_clause in with_clauses {
                    with_clause.query.remove_table_references(table);
                }

                // Recursively process subqueries
                for subquery in subqueries {
                    subquery.remove_table_references(table);
                }

                // Update the original query string
                let mut new_query = self.original.clone();
                // Remove table from FROM clause
                let from_pattern = format!(r",\s*{}\b|\b{}\s*,", table, table);
                new_query = Regex::new(&from_pattern)
                    .unwrap()
                    .replace_all(&new_query, "")
                    .to_string();
                self.original = new_query;
            }
            StatementKind::CreateView { name, query } => {
                query.remove_table_references(table);
                // Update the original query string to match the modified query
                self.original = format!("CREATE VIEW {} AS {}", name, query.original);
            }
            StatementKind::Unknown => {
                // Try to parse as a SELECT statement if it contains SELECT
                if self.original.to_uppercase().contains("SELECT") {
                    if let Ok(mut select_stmt) = parse_select_statement(&self.original) {
                        select_stmt.remove_table_references(table);
                        self.kind = select_stmt.kind;
                        self.original = select_stmt.original;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn get_tables(&self) -> Vec<&String> {
        match &self.kind {
            StatementKind::Select {
                tables, subqueries, ..
            } => {
                let mut all_tables = tables.iter().collect::<Vec<_>>();
                for subquery in subqueries {
                    all_tables.extend(subquery.get_tables());
                }
                all_tables
            }
            StatementKind::CreateView { query, .. } => query.get_tables(),
            _ => vec![],
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
        r"(?i)^\s*INSERT(?:\s+OR\s+(?:REPLACE|IGNORE))?\s+INTO\s+([^(]+)\s*\(\s*([^)]*)\s*\)\s*VALUES\s*\(\s*([^)]*)\s*\)",
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
    Ok(Statement::new_insert(
        query.to_string(),
        table,
        columns_and_values,
    ))
}

pub fn parse_create_table_statement(query: &str) -> Result<Statement, Box<dyn std::error::Error>> {
    // Compile the regex (case-insensitive, optional IF NOT EXISTS)
    let re = Regex::new(
        r"(?i)^\s*CREATE\s+TABLE(?:\s+IF\s+NOT\s+EXISTS)?\s+([^(]+)\s*\(\s*([^)]*)\s*\)",
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
            let ty = def[idx..].trim().to_string();
            columns.insert(col, ty);
        } else {
            // no whitespace? treat the whole thing as column name
            columns.insert(def.to_string(), String::new());
        }
    }

    // Return the parsed statement
    Ok(Statement::new_create_table(
        query.to_string(),
        name,
        columns,
    ))
}

pub fn parse_create_view_statement(query: &str) -> Result<Statement, Box<dyn std::error::Error>> {
    // Compile the regex (case-insensitive)
    let re = Regex::new(r"(?i)^\s*CREATE\s+VIEW\s+([^\s]+)\s+AS\s+(.+)$")?;

    // Try to match and capture:
    // 1 = view name, 2 = view definition
    let caps = re
        .captures(query)
        .ok_or_else(|| format!("Not a valid CREATE VIEW statement: {}", query))?;

    // 1) Extract & trim view name
    let name = caps[1].trim().to_string();

    // 2) Extract & trim view definition
    let view_def = caps[2].trim();

    // Parse the view definition as a statement
    let query_stmt = Statement::new(view_def);

    // Return the parsed statement
    Ok(Statement::new_create_view(
        query.to_string(),
        name,
        query_stmt,
    ))
}

pub fn parse_select_statement(query: &str) -> Result<Statement, Box<dyn std::error::Error>> {
    // First check for WITH clause
    let (with_clauses, remaining_query) = if query.to_uppercase().contains("WITH") {
        let re = Regex::new(r"(?i)^\s*WITH\s+(.+?)\s+AS\s+\((.+?)\)\s+(.+)$")?;
        if let Some(caps) = re.captures(query) {
            let cte_name = caps[1].trim().to_string();
            let cte_query = caps[2].trim();
            let main_query = caps[3].trim();

            // Parse the CTE query
            let cte_stmt = parse_select_statement(cte_query)?;

            // Parse the main query
            let main_stmt = parse_select_statement(main_query)?;

            // Combine them
            if let StatementKind::Select {
                mut columns,
                mut tables,
                mut conditions,
                mut subqueries,
                is_distinct,
                ..
            } = main_stmt.kind
            {
                let mut with_clauses = Vec::new();
                with_clauses.push(WithClause {
                    name: cte_name,
                    query: Box::new(cte_stmt),
                });

                return Ok(Statement::new_select(
                    query.to_string(),
                    with_clauses,
                    columns,
                    tables,
                    conditions,
                    subqueries,
                    is_distinct,
                ));
            }
        }
        (Vec::new(), query)
    } else {
        (Vec::new(), query)
    };

    // Parse the main SELECT statement
    let re = Regex::new(
        r"(?i)^\s*SELECT\s+(?:DISTINCT\s+)?(.+?)\s+FROM\s+(.+?)(?:\s+WHERE\s+(.+))?(?:\s*;)?$",
    )?;

    let caps = re
        .captures(remaining_query)
        .ok_or_else(|| format!("Not a valid SELECT statement: {}", remaining_query))?;

    let columns_str = caps[1].trim();
    let tables_str = caps[2].trim();
    let where_clause = caps.get(3).map(|m| m.as_str().trim());

    // Parse columns
    let columns = columns_str
        .split(',')
        .map(|col| {
            let parts: Vec<&str> = col.trim().split('.').collect();
            if parts.len() > 1 {
                Column {
                    table: Some(parts[0].trim().to_string()),
                    name: parts[1].trim().to_string(),
                }
            } else {
                Column {
                    table: None,
                    name: parts[0].trim().to_string(),
                }
            }
        })
        .collect();

    // Parse tables
    let tables = tables_str
        .split(',')
        .map(|t| t.trim().to_string())
        .collect();

    // Parse conditions
    let conditions = if let Some(where_str) = where_clause {
        where_str
            .split("AND")
            .map(|cond| {
                let parts: Vec<&str> = cond.trim().split('.').collect();
                if parts.len() > 1 {
                    Condition {
                        table: parts[0].trim().to_string(),
                        column: parts[1].trim().to_string(),
                        value: "".to_string(), // Simplified for now
                    }
                } else {
                    Condition {
                        table: "".to_string(),
                        column: parts[0].trim().to_string(),
                        value: "".to_string(),
                    }
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    Ok(Statement::new_select(
        query.to_string(),
        with_clauses,
        columns,
        tables,
        conditions,
        Vec::new(), // subqueries
        query.to_uppercase().contains("DISTINCT"),
    ))
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
            StatementKind::CreateTable {
                ref name,
                ref columns,
            } => {
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
