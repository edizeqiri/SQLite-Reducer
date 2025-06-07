use regex::Regex;
use std::collections::HashMap;

use crate::statements::types::{Column, Condition, Statement, StatementKind, WithClause};

pub fn parse_insert_statement(query: &str) -> Result<Statement, Box<dyn std::error::Error>> {
    let re = Regex::new(
        r"(?i)^\s*INSERT(?:\s+OR\s+(?:REPLACE|IGNORE))?\s+INTO\s+([^(]+)\s*\(\s*([^)]*)\s*\)\s*VALUES\s*\(\s*([^)]*)\s*\)",
    )?;

    let caps = re
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

    let col_values: Vec<String> = caps[3]
        .trim()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    let mut columns_and_values = HashMap::new();
    for (name, value) in col_names.iter().zip(col_values.iter()) {
        columns_and_values.insert(name.clone(), value.clone());
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

    let re = Regex::new(
        r"(?i)^\s*SELECT\s+(?:DISTINCT\s+)?(.+?)\s+FROM\s+(.+?)(?:\s+WHERE\s+(.+))?(?:\s*;)?$",
    )?;

    let caps = re
        .captures(remaining_query)
        .ok_or_else(|| format!("Not a valid SELECT statement: {}", remaining_query))?;

    let columns_str = caps[1].trim();
    let tables_str = caps[2].trim();
    let where_clause = caps.get(3).map(|m| m.as_str().trim());

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

    let tables = tables_str
        .split(',')
        .map(|t| t.trim().to_string())
        .collect();

    let conditions = if let Some(where_str) = where_clause {
        where_str
            .split("AND")
            .map(|cond| {
                let parts: Vec<&str> = cond.trim().split('.').collect();
                if parts.len() > 1 {
                    Condition {
                        table: parts[0].trim().to_string(),
                        column: parts[1].trim().to_string(),
                        value: "".to_string(),
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

    Ok(Statement {
        original: query.to_string(),
        kind: StatementKind::Select {
            with_clauses,
            columns,
            tables,
            conditions,
            subqueries: Vec::new(),
            is_distinct: query.to_uppercase().contains("DISTINCT"),
        },
    })
}
