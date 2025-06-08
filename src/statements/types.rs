use std::collections::HashMap;

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
        columns: Vec<Column>,
    },
    Insert {
        table: String,
        columns: Vec<String>,
        values: Vec<Vec<String>>,
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
    Trigger {
        name: String,
        timing: String,
        event: String,
        table: String,
        body: Vec<String>,
    },
    Drop,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub original: String,
    pub kind: StatementKind,
}
