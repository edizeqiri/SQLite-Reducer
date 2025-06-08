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

                // Process subqueries
                subqueries.retain_mut(|subquery| {
                    subquery.remove_table_references(table);
                    !subquery.get_tables().is_empty()
                });

                // Update the original SQL query
                let mut updated_query = self.original.clone();
                
                // Remove table from FROM clause
                let from_pattern = format!(r",\s*{}\b|\b{}\s*,", table, table);
                updated_query = Regex::new(&from_pattern)
                    .unwrap()
                    .replace_all(&updated_query, "")
                    .to_string();
                
                // Remove table references from WHERE clause
                let where_pattern = format!(r"{}\.\w+", table);
                updated_query = Regex::new(&where_pattern)
                    .unwrap()
                    .replace_all(&updated_query, "")
                    .to_string();
                
                // Remove table references from GROUP BY clause
                let group_by_pattern = format!(r",\s*{}\.\w+|\b{}\.\w+\s*,", table, table);
                updated_query = Regex::new(&group_by_pattern)
                    .unwrap()
                    .replace_all(&updated_query, "")
                    .to_string();
                
                // Remove table references from HAVING clause
                let having_pattern = format!(r"{}\.\w+", table);
                updated_query = Regex::new(&having_pattern)
                    .unwrap()
                    .replace_all(&updated_query, "")
                    .to_string();
                
                // Remove table references from ORDER BY clause
                let order_by_pattern = format!(r",\s*{}\.\w+|\b{}\.\w+\s*,", table, table);
                updated_query = Regex::new(&order_by_pattern)
                    .unwrap()
                    .replace_all(&updated_query, "")
                    .to_string();

                // Clean up any double commas or spaces
                updated_query = Regex::new(r",\s*,")
                    .unwrap()
                    .replace_all(&updated_query, ",")
                    .to_string();
                updated_query = Regex::new(r"\s+")
                    .unwrap()
                    .replace_all(&updated_query, " ")
                    .to_string();

                self.original = updated_query;
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
