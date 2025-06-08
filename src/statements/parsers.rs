use regex::Regex;
use std::collections::HashMap;

use crate::statements::types::{Column, Condition, Statement, StatementKind, WithClause};

pub fn parse_insert_statement(query: &str) -> Result<Statement, Box<dyn std::error::Error>> {
    // First, extract the table name and column list
    let table_re = Regex::new(
        r"(?i)^\s*INSERT(?:\s+OR\s+(?:REPLACE|IGNORE|FAIL))?\s+INTO\s+([^(]+)\s*\(\s*([^)]*)\s*\)",
    )?;

    let caps = table_re
        .captures(query)
        .ok_or_else(|| format!("Not a valid INSERT statement: {}", query))?;

    let table = caps[1].trim().to_string();
    let col_names: Vec<String> = caps[2]
        .trim()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    // Now extract all VALUES clauses
    let values_re = Regex::new(r"(?i)VALUES\s*\(\s*([^)]*)\s*\)(?:\s*,\s*\(\s*([^)]*)\s*\))*")?;
    let values_caps = values_re
        .captures(query)
        .ok_or_else(|| format!("No VALUES clause found in INSERT statement: {}", query))?;

    // Get all value groups
    let mut all_values = Vec::new();
    for i in 1..values_caps.len() {
        if let Some(values_str) = values_caps.get(i) {
            let values: Vec<String> = values_str
                .as_str()
                .split(',')
                .map(str::trim)
                .map(str::to_string)
                .collect();
            if !values.is_empty() {
                all_values.push(values);
            }
        }
    }

    // Create a HashMap with the first row of values
    let mut columns_and_values = HashMap::new();
    if !all_values.is_empty() {
        for (name, value) in col_names.iter().zip(all_values[0].iter()) {
            columns_and_values.insert(name.clone(), value.clone());
        }
    }

    Ok(Statement {
        original: query.to_string(),
        kind: StatementKind::Insert {
            table,
            columns_and_values,
        },
    })
}

pub fn parse_create_table_statement(query: &str) -> Result<Statement, Box<dyn std::error::Error>> {
    let re = Regex::new(
        r"(?i)^\s*CREATE\s+TABLE(?:\s+IF\s+NOT\s+EXISTS)?\s+([^(]+)\s*\(\s*([^)]*)\s*\)",
    )?;

    let caps = re
        .captures(query)
        .ok_or_else(|| format!("Not a valid CREATE TABLE statement: {}", query))?;

    let name = caps[1].trim().to_string();
    let cols_block = caps[2].trim();

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
            columns.insert(def.to_string(), String::new());
        }
    }

    Ok(Statement {
        original: query.to_string(),
        kind: StatementKind::CreateTable { name, columns },
    })
}

pub fn parse_create_view_statement(query: &str) -> Result<Statement, Box<dyn std::error::Error>> {
    let re = Regex::new(r"(?i)^\s*CREATE\s+VIEW\s+([^\s]+)\s+AS\s+(.+)$")?;

    let caps = re
        .captures(query)
        .ok_or_else(|| format!("Not a valid CREATE VIEW statement: {}", query))?;

    let name = caps[1].trim().to_string();
    let view_def = caps[2].trim();
    let query_stmt = Statement {
        original: view_def.to_string(),
        kind: StatementKind::Unknown,
    };

    Ok(Statement {
        original: query.to_string(),
        kind: StatementKind::CreateView {
            name,
            query: Box::new(query_stmt),
        },
    })
}

pub fn parse_select_statement(query: &str) -> Result<Statement, Box<dyn std::error::Error>> {
    // Handle WITH clauses first
    let (with_clauses, remaining_query) = if query.to_uppercase().contains("WITH") {
        let re = Regex::new(r"(?i)^\s*WITH\s+(.+?)\s+AS\s+\((.+?)\)\s+(.+)$")?;
        if let Some(caps) = re.captures(query) {
            let cte_name = caps[1].trim().to_string();
            let cte_query = caps[2].trim();
            let main_query = caps[3].trim();

            let cte_stmt = parse_select_statement(cte_query)?;
            let main_stmt = parse_select_statement(main_query)?;

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

                return Ok(Statement {
                    original: query.to_string(),
                    kind: StatementKind::Select {
                        with_clauses,
                        columns,
                        tables,
                        conditions,
                        subqueries,
                        is_distinct,
                    },
                });
            }
        }
        (Vec::new(), query)
    } else {
        (Vec::new(), query)
    };

    // Handle EXISTS subqueries
    let exists_re = Regex::new(r"(?i)EXISTS\s*\(\s*(SELECT.+?)\s*\)")?;
    let mut subqueries = Vec::new();
    let mut processed_query = remaining_query.to_string();

    for cap in exists_re.captures_iter(remaining_query) {
        let subquery = cap[1].trim();
        if let Ok(substmt) = parse_select_statement(subquery) {
            subqueries.push(substmt);
        }
        processed_query = processed_query.replace(&cap[0], "EXISTS_SUBQUERY");
    }

    // Parse the main SELECT statement with a more flexible pattern
    let re = Regex::new(
        r"(?i)^\s*SELECT\s+(?:DISTINCT\s+)?(.+?)(?:\s+FROM\s+)([^;]+?)(?:\s+WHERE\s+(.+?))?(?:\s+GROUP\s+BY\s+(.+?))?(?:\s+HAVING\s+(.+?))?(?:\s+ORDER\s+BY\s+(.+?))?(?:\s+LIMIT\s+(\d+))?(?:\s*;)?$",
    )?;

    let caps = re
        .captures(&processed_query)
        .ok_or_else(|| format!("Not a valid SELECT statement: {}", processed_query))?;

    // Parse columns
    let columns_str = caps[1].trim();
    let columns = if columns_str.contains("EXISTS_SUBQUERY") {
        vec![Column {
            table: None,
            name: columns_str.trim().to_string(),
        }]
    } else {
        columns_str
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
            .collect()
    };

    // Parse tables with better handling of JOINs and aliases
    let tables_str = caps[2].trim();
    let tables = tables_str
        .split(',')
        .flat_map(|t| {
            let t = t.trim();
            // Handle JOIN clauses
            if t.to_uppercase().contains("JOIN") {
                let join_parts: Vec<&str> = t.split_whitespace().collect();
                join_parts
                    .iter()
                    .filter(|&&part| {
                        !part.to_uppercase().contains("JOIN") && !part.to_uppercase().contains("ON")
                    })
                    .map(|&part| part.trim().to_string())
                    .collect::<Vec<String>>()
            } else {
                // Handle simple table references and aliases
                if let Some(idx) = t.find(char::is_whitespace) {
                    vec![t[..idx].trim().to_string()]
                } else {
                    vec![t.to_string()]
                }
            }
        })
        .collect();

    // Parse conditions with better table reference detection
    let mut conditions = Vec::new();
    if let Some(where_str) = caps.get(3).map(|m| m.as_str().trim()) {
        // Find all table references in the WHERE clause
        let table_re = Regex::new(r"(?i)([a-zA-Z_][a-zA-Z0-9_]*)\.([a-zA-Z_][a-zA-Z0-9_]*)")?;
        let mut table_refs = Vec::new();

        for cap in table_re.captures_iter(where_str) {
            let table_name = cap[1].trim().to_string();
            if !table_refs.contains(&table_name) {
                table_refs.push(table_name);
            }
        }

        // For each table reference, create a condition
        for table_name in table_refs {
            conditions.push(Condition {
                table: table_name,
                column: where_str.to_string(),
                value: "".to_string(),
            });
        }
    }

    Ok(Statement {
        original: query.to_string(),
        kind: StatementKind::Select {
            with_clauses,
            columns,
            tables,
            conditions,
            subqueries,
            is_distinct: query.to_uppercase().contains("DISTINCT"),
        },
    })
}
