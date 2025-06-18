use regex::Regex;
use std::fmt;

use crate::statements::types::{Column, Statement, StatementKind};

impl Statement {
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
                is_distinct,
            } => {
                // If this is the only table in the FROM clause, replace the entire SELECT with SELECT 1
                if tables.len() == 1 && tables[0] == table {
                    self.kind = StatementKind::Select {
                        with_clauses: Vec::new(),
                        columns: vec![Column {
                            table: None,
                            name: "1".to_string(),
                        }],
                        tables: Vec::new(),
                        conditions: Vec::new(),
                        subqueries: Vec::new(),
                        is_distinct: false,
                    };
                    self.original = "SELECT 1".to_string();
                    return;
                }

                // Remove table from tables list
                tables.retain(|t| t != table);

                // If no tables remain, replace with SELECT 1
                if tables.is_empty() {
                    self.kind = StatementKind::Select {
                        with_clauses: Vec::new(),
                        columns: vec![Column {
                            table: None,
                            name: "1".to_string(),
                        }],
                        tables: Vec::new(),
                        conditions: Vec::new(),
                        subqueries: Vec::new(),
                        is_distinct: false,
                    };
                    self.original = "SELECT 1".to_string();
                    return;
                }

                columns.retain(|col| col.table.as_ref() != Some(&table.to_string()));

                conditions.retain(|cond| cond.table != table);

                for with_clause in with_clauses.iter_mut() {
                    with_clause.query.remove_table_references(table);
                }

                subqueries.retain_mut(|subquery| {
                    subquery.remove_table_references(table);
                    !subquery.get_tables().is_empty()
                });

                let mut parts = Vec::new();

                if !with_clauses.is_empty() {
                    let with_parts: Vec<String> = with_clauses
                        .iter()
                        .map(|wc| format!("{} AS ({})", wc.name, wc.query.original))
                        .collect();
                    parts.push(format!("WITH {}", with_parts.join(", ")));
                }

                let distinct_str = if *is_distinct { "DISTINCT " } else { "" };
                let columns_str = columns
                    .iter()
                    .map(|col| {
                        if let Some(tbl) = &col.table {
                            format!("{}.{}", tbl, col.name)
                        } else {
                            col.name.clone()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                parts.push(format!("SELECT {}{}", distinct_str, columns_str));

                if !tables.is_empty() {
                    parts.push(format!("FROM {}", tables.join(", ")));
                }

                if !conditions.is_empty() {
                    let where_conditions: Vec<String> =
                        conditions.iter().map(|cond| cond.column.clone()).collect();
                    parts.push(format!("WHERE {}", where_conditions.join(" AND ")));
                }

                let group_by_cols: Vec<String> = columns
                    .iter()
                    .filter(|col| col.table.as_ref() != Some(&table.to_string()))
                    .map(|col| col.name.clone())
                    .collect();
                if !group_by_cols.is_empty() {
                    parts.push(format!("GROUP BY {}", group_by_cols.join(", ")));
                }

                if !conditions.is_empty() {
                    let having_conditions: Vec<String> =
                        conditions.iter().map(|cond| cond.column.clone()).collect();
                    parts.push(format!("HAVING {}", having_conditions.join(" AND ")));
                }

                let order_by_cols: Vec<String> = columns
                    .iter()
                    .filter(|col| col.table.as_ref() != Some(&table.to_string()))
                    .map(|col| col.name.clone())
                    .collect();
                if !order_by_cols.is_empty() {
                    parts.push(format!("ORDER BY {}", order_by_cols.join(", ")));
                }

                self.original = parts.join(" ");
            }
            StatementKind::Trigger {
                name,
                timing,
                event,
                table: trigger_table,
                body,
            } => {
                if trigger_table == table {
                    self.kind = StatementKind::Unknown;
                    self.original = "".to_string();
                    return;
                }

                if body.iter().any(|stmt| stmt.contains(table)) {
                    self.kind = StatementKind::Unknown;
                    self.original = "".to_string();
                    return;
                }
            }
            StatementKind::CreateView { name, query } => {
                query.remove_table_references(table);
                self.original = format!("CREATE VIEW {} AS {}", name, query.original);
            }
            StatementKind::Unknown => {
                if self.original.to_uppercase().contains("CREATE TRIGGER") {
                    if let Ok(mut trigger_stmt) =
                        crate::statements::parsers::parse_trigger_statement(&self.original)
                    {
                        trigger_stmt.remove_table_references(table);
                        if !trigger_stmt.original.is_empty() {
                            self.kind = trigger_stmt.kind;
                            self.original = trigger_stmt.original;
                        } else {
                            self.original = "".to_string();
                        }
                    }
                } else if self.original.to_uppercase().contains("SELECT") {
                    if let Ok(mut select_stmt) =
                        crate::statements::parsers::parse_select_statement(&self.original)
                    {
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

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.original)
    }
}
