use regex::Regex;
use std::collections::HashMap;

use crate::statements::types::{Column, Condition, Statement, StatementKind, WithClause};

pub fn parse_insert_statement(sql: &str) -> Result<Statement, Box<dyn std::error::Error>> {
    let insert_regex = Regex::new(
        r"(?i)INSERT\s+(?:OR\s+(?:FAIL|IGNORE|REPLACE)\s+)?INTO\s+(\w+)\s*(?:\((.*?)\))?\s*VALUES\s*(.*)",
    )?;

    if let Some(caps) = insert_regex.captures(sql) {
        let table = caps[1].to_string();
        let columns_str = caps.get(2).map_or("", |m| m.as_str());
        let values_str = caps[3].to_string();

        let columns: Vec<String> = if !columns_str.is_empty() {
            columns_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            Vec::new()
        };

        let values: Vec<Vec<String>> = values_str
            .split("),")
            .map(|value_group| {
                value_group
                    .trim_start_matches("(")
                    .trim_end_matches(")")
                    .trim()
                    .split(",")
                    .map(|v| v.trim().to_string())
                    .collect()
            })
            .collect();

        Ok(Statement {
            original: sql.to_string(),
            kind: StatementKind::Insert {
                table,
                columns,
                values,
            },
        })
    } else {
        Ok(Statement {
            original: sql.to_string(),
            kind: StatementKind::Unknown,
        })
    }
}

pub fn parse_create_table(sql: &str) -> Result<Statement, Box<dyn std::error::Error>> {
    let create_table_regex =
        Regex::new(r"(?i)CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?(\w+)\s*\((.*?)\)")?;

    if let Some(caps) = create_table_regex.captures(sql) {
        let name = caps[1].to_string();
        let columns_str = caps[2].to_string();

        let columns: Vec<Column> = columns_str
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|col_str| {
                let parts: Vec<&str> = col_str.trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    Column {
                        table: None,
                        name: parts[0].to_string(),
                    }
                } else {
                    Column {
                        table: None,
                        name: col_str.trim().to_string(),
                    }
                }
            })
            .collect();

        Ok(Statement {
            original: sql.to_string(),
            kind: StatementKind::CreateTable { name, columns },
        })
    } else {
        Err("Not a CREATE TABLE statement".into())
    }
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

    let re = Regex::new(
        r"(?i)^\s*SELECT\s+(?:DISTINCT\s+)?(.+?)(?:\s+FROM\s+)([^;]+?)(?:\s+WHERE\s+(.+?))?(?:\s+GROUP\s+BY\s+(.+?))?(?:\s+HAVING\s+(.+?))?(?:\s+ORDER\s+BY\s+(.+?))?(?:\s+LIMIT\s+(\d+))?(?:\s*;)?$",
    )?;

    let caps = re
        .captures(&processed_query)
        .ok_or_else(|| format!("Not a valid SELECT statement: {}", processed_query))?;

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

    let tables_str = caps[2].trim();
    let tables = tables_str
        .split(',')
        .flat_map(|t| {
            let t = t.trim();
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
                if let Some(idx) = t.find(char::is_whitespace) {
                    vec![t[..idx].trim().to_string()]
                } else {
                    vec![t.to_string()]
                }
            }
        })
        .collect();

    let mut conditions = Vec::new();
    if let Some(where_str) = caps.get(3).map(|m| m.as_str().trim()) {
        let table_re = Regex::new(r"(?i)([a-zA-Z_][a-zA-Z0-9_]*)\.([a-zA-Z_][a-zA-Z0-9_]*)")?;
        let mut table_refs = Vec::new();

        for cap in table_re.captures_iter(where_str) {
            let table_name = cap[1].trim().to_string();
            if !table_refs.contains(&table_name) {
                table_refs.push(table_name);
            }
        }

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

pub fn parse_trigger_statement(sql: &str) -> Result<Statement, Box<dyn std::error::Error>> {
    let trigger_regex = Regex::new(
        r"(?i)CREATE\s+TRIGGER\s+(\w+)\s+(BEFORE|AFTER|INSTEAD\s+OF)\s+(INSERT|UPDATE|DELETE)\s+ON\s+(\w+)\s+BEGIN\s+(.*?)(?:END|$)",
    )?;

    if let Some(caps) = trigger_regex.captures(sql) {
        let name = caps[1].to_string();
        let timing = caps[2].to_string();
        let event = caps[3].to_string();
        let table = caps[4].to_string();
        let body = caps[5].to_string();

        let body_statements: Vec<String> = body
            .split(';')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let mut parsed_body = Vec::new();
        for stmt in body_statements {
            if stmt.to_uppercase().starts_with("UPDATE") {
                if let Ok(update_stmt) = parse_update_statement(&stmt) {
                    parsed_body.push(update_stmt.original);
                    continue;
                }
            }
            parsed_body.push(stmt);
        }

        Ok(Statement {
            original: sql.to_string(),
            kind: StatementKind::Trigger {
                name,
                timing,
                event,
                table,
                body: parsed_body,
            },
        })
    } else {
        Ok(Statement {
            original: sql.to_string(),
            kind: StatementKind::Unknown,
        })
    }
}

pub fn parse_update_statement(sql: &str) -> Result<Statement, Box<dyn std::error::Error>> {
    let update_regex = Regex::new(r"(?i)^\s*UPDATE\s+(\w+)")?;

    if let Some(caps) = update_regex.captures(sql) {
        let table = caps[1].to_string();

        Ok(Statement {
            original: sql.to_string(),
            kind: StatementKind::Update {
                table,
                set_clauses: Vec::new(),
                where_clause: None,
            },
        })
    } else {
        Ok(Statement {
            original: sql.to_string(),
            kind: StatementKind::Unknown,
        })
    }
}

pub fn parse_delete_statement(sql: &str) -> Result<Statement, Box<dyn std::error::Error>> {
    let delete_regex = Regex::new(r"(?i)^\s*DELETE\s+FROM\s+(\w+)(?:\s+WHERE\s+(.+))?(?:\s*;)?$")?;

    if let Some(caps) = delete_regex.captures(sql) {
        let table = caps[1].to_string();
        let where_clause = caps.get(2).map(|m| m.as_str().trim().to_string());

        Ok(Statement {
            original: sql.to_string(),
            kind: StatementKind::Delete {
                table,
                where_clause,
            },
        })
    } else {
        Ok(Statement {
            original: sql.to_string(),
            kind: StatementKind::Unknown,
        })
    }
}

pub fn parse_alter_table_statement(sql: &str) -> Result<Statement, Box<dyn std::error::Error>> {
    let alter_regex = Regex::new(r"(?i)^\s*ALTER\s+TABLE\s+(\w+)\s+(.+?)(?:\s*;)?$")?;

    if let Some(caps) = alter_regex.captures(sql) {
        let table = caps[1].to_string();
        let operation = caps[2].trim().to_string();

        Ok(Statement {
            original: sql.to_string(),
            kind: StatementKind::AlterTable { table, operation },
        })
    } else {
        Ok(Statement {
            original: sql.to_string(),
            kind: StatementKind::Unknown,
        })
    }
}
