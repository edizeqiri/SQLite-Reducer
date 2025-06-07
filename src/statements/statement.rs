use regex::Regex;
use std::fmt;

use crate::statements::types::{Statement, StatementKind};

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
                ..
            } => {
                tables.retain(|t| t != table);
                columns.retain(|col| col.table.as_ref() != Some(&table.to_string()));
                conditions.retain(|cond| cond.table != table);

                for with_clause in with_clauses {
                    with_clause.query.remove_table_references(table);
                }

                for subquery in subqueries {
                    subquery.remove_table_references(table);
                }

                let from_pattern = format!(r",\s*{}\b|\b{}\s*,", table, table);
                self.original = Regex::new(&from_pattern)
                    .unwrap()
                    .replace_all(&self.original, "")
                    .to_string();
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
