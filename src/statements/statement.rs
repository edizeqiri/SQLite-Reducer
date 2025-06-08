use regex::Regex;
use std::fmt;

use crate::statements::types::{Statement, StatementKind, Column};

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

                // Remove columns referencing the table
                columns.retain(|col| col.table.as_ref() != Some(&table.to_string()));

                // Remove conditions referencing the table
                conditions.retain(|cond| cond.table != table);

                // Process WITH clauses
                for with_clause in with_clauses.iter_mut() {
                    with_clause.query.remove_table_references(table);
                }

                // Process subqueries
                subqueries.retain_mut(|subquery| {
                    subquery.remove_table_references(table);
                    !subquery.get_tables().is_empty()
                });

                // Reconstruct the SQL query
                let mut parts = Vec::new();

                // Add WITH clauses if any
                if !with_clauses.is_empty() {
                    let with_parts: Vec<String> = with_clauses
                        .iter()
                        .map(|wc| format!("{} AS ({})", wc.name, wc.query.original))
                        .collect();
                    parts.push(format!("WITH {}", with_parts.join(", ")));
                }

                // Add SELECT part
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

                // Add FROM part
                if !tables.is_empty() {
                    parts.push(format!("FROM {}", tables.join(", ")));
                }

                // Add WHERE part
                if !conditions.is_empty() {
                    let where_conditions: Vec<String> =
                        conditions.iter().map(|cond| cond.column.clone()).collect();
                    parts.push(format!("WHERE {}", where_conditions.join(" AND ")));
                }

                // Add GROUP BY part if any columns remain
                let group_by_cols: Vec<String> = columns
                    .iter()
                    .filter(|col| col.table.as_ref() != Some(&table.to_string()))
                    .map(|col| col.name.clone())
                    .collect();
                if !group_by_cols.is_empty() {
                    parts.push(format!("GROUP BY {}", group_by_cols.join(", ")));
                }

                // Add HAVING part if any conditions remain
                if !conditions.is_empty() {
                    let having_conditions: Vec<String> =
                        conditions.iter().map(|cond| cond.column.clone()).collect();
                    parts.push(format!("HAVING {}", having_conditions.join(" AND ")));
                }

                // Add ORDER BY part if any columns remain
                let order_by_cols: Vec<String> = columns
                    .iter()
                    .filter(|col| col.table.as_ref() != Some(&table.to_string()))
                    .map(|col| col.name.clone())
                    .collect();
                if !order_by_cols.is_empty() {
                    parts.push(format!("ORDER BY {}", order_by_cols.join(", ")));
                }

                // Join all parts with spaces
                self.original = parts.join(" ");
            }
            StatementKind::CreateView { name, query } => {
                query.remove_table_references(table);
                self.original = format!("CREATE VIEW {} AS {}", name, query.original);
            }
            StatementKind::Unknown => {
                if self.original.to_uppercase().contains("SELECT") {
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
