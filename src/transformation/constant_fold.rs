use crate::driver::test_query;
use crate::transformation::transformer::Transform;
use sqlparser::ast::Value::{Boolean, Number};
use sqlparser::ast::{
    BinaryOperator, Expr as SQLExpr, Query, SelectItem, SetExpr, Statement, UnaryOperator,
    ValueWithSpan,
};


#[derive(Debug, Default)]
pub struct ConstantFold;

impl Transform for ConstantFold {
    fn apply(&self, stmt: Statement) -> Statement {
        fold_statement(stmt).unwrap_or_else(|| create_empty_query())
    }
}

fn create_empty_query() -> Statement {
    Statement::Query(Box::new(Query {
        body: Box::new(SetExpr::Values(sqlparser::ast::Values {
            explicit_row: false,
            rows: vec![],
        })),
        with: None,
        order_by: None,
        limit_clause: None,
        fetch: None,
        locks: vec![],
        for_clause: None,
        settings: None,
        format_clause: None,
    }))
}

fn fold_statement(stmt: Statement) -> Option<Statement> {
    match stmt {
        Statement::Query(boxed_q) => {
            let q = *boxed_q;
            let new_body = fold_setexpr((*q.body).clone());

            if let SetExpr::Select(select) = &new_body {
                if let Some(SQLExpr::Value(ValueWithSpan {
                    value: Boolean(false),
                    ..
                })) = select.selection
                {
                    return None;
                }
            }

            let folded_query = Statement::Query(Box::new(Query {
                body: Box::new(new_body.clone()),
                with: q.with.clone(),
                order_by: q.order_by.clone(),
                limit_clause: q.limit_clause.clone(),
                fetch: q.fetch.clone(),
                locks: q.locks.clone(),
                for_clause: q.for_clause.clone(),
                settings: q.settings.clone(),
                format_clause: q.format_clause.clone(),
            }));

            let query_str = folded_query.to_string();
            match test_query(&query_str) {
                Ok(false) => Some(folded_query),
                _ => {
                    let original_query = Statement::Query(Box::new(Query {
                        body: Box::new((*q.body).clone()),
                        with: q.with.clone(),
                        order_by: q.order_by.clone(),
                        limit_clause: q.limit_clause.clone(),
                        fetch: q.fetch.clone(),
                        locks: q.locks.clone(),
                        for_clause: q.for_clause.clone(),
                        settings: q.settings.clone(),
                        format_clause: q.format_clause.clone(),
                    }));
                    let original_str = original_query.to_string();
                    match test_query(&original_str) {
                        Ok(false) => Some(original_query),
                        _ => None,
                    }
                }
            }
        }
        Statement::Insert(mut boxed_i) => {
            if let Some(source) = boxed_i.source.take() {
                let folded_body = fold_setexpr((*source.body).clone());

                let folded_source = Query {
                    body: Box::new(folded_body.clone()),
                    with: source.with.clone(),
                    order_by: source.order_by.clone(),
                    limit_clause: source.limit_clause.clone(),
                    fetch: source.fetch.clone(),
                    locks: source.locks.clone(),
                    for_clause: source.for_clause.clone(),
                    settings: source.settings.clone(),
                    format_clause: source.format_clause.clone(),
                };
                let mut folded_insert = boxed_i.clone();
                folded_insert.source = Some(Box::new(folded_source.clone()));
                let folded_query = Statement::Insert(folded_insert);

                let query_str = folded_query.to_string();
                match test_query(&query_str) {
                    Ok(false) => {
                        boxed_i.source = Some(Box::new(folded_source));
                        Some(Statement::Insert(boxed_i))
                    }
                    _ => {
                        let original_source = Query {
                            body: Box::new((*source.body).clone()),
                            with: source.with.clone(),
                            order_by: source.order_by.clone(),
                            limit_clause: source.limit_clause.clone(),
                            fetch: source.fetch.clone(),
                            locks: source.locks.clone(),
                            for_clause: source.for_clause.clone(),
                            settings: source.settings.clone(),
                            format_clause: source.format_clause.clone(),
                        };
                        let mut original_insert = boxed_i.clone();
                        original_insert.source = Some(Box::new(original_source.clone()));
                        let original_query = Statement::Insert(original_insert);
                        let original_str = original_query.to_string();
                        match test_query(&original_str) {
                            Ok(false) => {
                                boxed_i.source = Some(Box::new(original_source));
                                Some(Statement::Insert(boxed_i))
                            }
                            _ => None,
                        }
                    }
                }
            } else {
                Some(Statement::Insert(boxed_i))
            }
        }
        other => Some(other),
    }
}

fn fold_setexpr(setexpr: SetExpr) -> SetExpr {
    match setexpr {
        SetExpr::Select(mut select) => {
            select.projection = fold_projection(select.projection);

            select.from = fold_from_clause(select.from);

            if let Some(expr) = select.selection {
                select.selection = Some(fold_expr(expr));
            }

            SetExpr::Select(select)
        }
        SetExpr::Values(values) => {
            let folded_values = values
                .rows
                .into_iter()
                .map(|row| row.into_iter().map(fold_expr).collect())
                .collect();
            SetExpr::Values(sqlparser::ast::Values {
                rows: folded_values,
                ..values
            })
        }
        other => other,
    }
}

fn fold_projection(projection: Vec<SelectItem>) -> Vec<SelectItem> {
    projection
        .into_iter()
        .map(|item| match item {
            SelectItem::UnnamedExpr(expr) => SelectItem::UnnamedExpr(fold_expr(expr)),
            other => other,
        })
        .collect()
}

fn fold_from_clause(
    from: Vec<sqlparser::ast::TableWithJoins>,
) -> Vec<sqlparser::ast::TableWithJoins> {
    from.into_iter()
        .map(|table_with_joins| match table_with_joins.relation {
            sqlparser::ast::TableFactor::Derived {
                lateral,
                subquery,
                alias,
            } => sqlparser::ast::TableWithJoins {
                relation: sqlparser::ast::TableFactor::Derived {
                    lateral,
                    subquery: Box::new(Query {
                        body: Box::new(fold_setexpr(*subquery.body)),
                        ..*subquery
                    }),
                    alias,
                },
                joins: table_with_joins.joins,
            },
            _ => table_with_joins,
        })
        .collect()
}

fn fold_numeric_value(value: f64) -> SQLExpr {
    if value.fract() == 0.0 {
        SQLExpr::Value(ValueWithSpan {
            value: Number(format!("{}", value.trunc() as i64), false),
            span: sqlparser::tokenizer::Span::empty(),
        })
    } else {
        SQLExpr::Value(ValueWithSpan {
            value: Number(format!("{}", value), false),
            span: sqlparser::tokenizer::Span::empty(),
        })
    }
}

fn fold_unary_op(op: UnaryOperator, expr: SQLExpr) -> SQLExpr {
    let inner = fold_expr(expr);

    match op {
        UnaryOperator::Not => match inner {
            SQLExpr::Value(ValueWithSpan {
                value: Boolean(b), ..
            }) => SQLExpr::Value(ValueWithSpan {
                value: Boolean(!b),
                span: sqlparser::tokenizer::Span::empty(),
            }),
            SQLExpr::UnaryOp {
                op: UnaryOperator::Not,
                expr: inner_boxed,
            } => fold_expr(*inner_boxed),
            _ => SQLExpr::UnaryOp {
                op,
                expr: Box::new(inner),
            },
        },
        UnaryOperator::Plus => inner,
        UnaryOperator::Minus => {
            if let SQLExpr::Value(ValueWithSpan {
                value: Number(n, _),
                ..
            }) = inner
            {
                if let Ok(num) = n.parse::<f64>() {
                    fold_numeric_value(-num)
                } else {
                    SQLExpr::UnaryOp {
                        op,
                        expr: Box::new(SQLExpr::Value(ValueWithSpan {
                            value: Number(n.clone(), false),
                            span: sqlparser::tokenizer::Span::empty(),
                        })),
                    }
                }
            } else {
                SQLExpr::UnaryOp {
                    op,
                    expr: Box::new(inner),
                }
            }
        }
        _ => SQLExpr::UnaryOp {
            op,
            expr: Box::new(inner),
        },
    }
}

fn fold_binary_op(left: SQLExpr, op: BinaryOperator, right: SQLExpr) -> SQLExpr {
    let l = fold_expr(left);
    let r = fold_expr(right);

    if let (
        SQLExpr::Value(ValueWithSpan {
            value: Boolean(lb), ..
        }),
        SQLExpr::Value(ValueWithSpan {
            value: Boolean(rb), ..
        }),
    ) = (&l, &r)
    {
        let bool_result = match op {
            BinaryOperator::Eq => *lb == *rb,
            BinaryOperator::NotEq => *lb != *rb,
            BinaryOperator::And => *lb && *rb,
            BinaryOperator::Or => *lb || *rb,
            _ => {
                return SQLExpr::BinaryOp {
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                }
            }
        };
        return SQLExpr::Value(Boolean(bool_result).with_empty_span());
    }

    if let (
        SQLExpr::Value(ValueWithSpan {
            value: Number(ls, _),
            ..
        }),
        SQLExpr::Value(ValueWithSpan {
            value: Number(rs, _),
            ..
        }),
    ) = (&l, &r)
    {
        if let (Ok(ln), Ok(rn)) = (ls.parse::<f64>(), rs.parse::<f64>()) {
            match op {
                BinaryOperator::Plus => return fold_numeric_value(ln + rn),
                BinaryOperator::Minus => return fold_numeric_value(ln - rn),
                BinaryOperator::Multiply => return fold_numeric_value(ln * rn),
                BinaryOperator::Divide => return fold_numeric_value(ln / rn),
                BinaryOperator::Eq => return SQLExpr::Value(Boolean(ln == rn).with_empty_span()),
                BinaryOperator::NotEq => {
                    return SQLExpr::Value(Boolean(ln != rn).with_empty_span())
                }
                BinaryOperator::Gt => return SQLExpr::Value(Boolean(ln > rn).with_empty_span()),
                BinaryOperator::Lt => return SQLExpr::Value(Boolean(ln < rn).with_empty_span()),
                BinaryOperator::GtEq => return SQLExpr::Value(Boolean(ln >= rn).with_empty_span()),
                BinaryOperator::LtEq => return SQLExpr::Value(Boolean(ln <= rn).with_empty_span()),
                _ => {}
            }
        }
    }

    SQLExpr::BinaryOp {
        left: Box::new(l),
        op,
        right: Box::new(r),
    }
}

fn fold_expr(expr: SQLExpr) -> SQLExpr {
    match expr {
        SQLExpr::UnaryOp {
            op,
            expr: boxed_expr,
        } => fold_unary_op(op, *boxed_expr),
        SQLExpr::Nested(inner) => {
            let folded = fold_expr(*inner);
            if let SQLExpr::Value(_) = folded {
                folded
            } else {
                SQLExpr::Nested(Box::new(folded))
            }
        }
        SQLExpr::BinaryOp { left, op, right } => fold_binary_op(*left, op, *right),
        other => other,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{driver, parser};

    #[test]
    fn test_fold_query1() {
        let query = "SELECT * FROM (VALUES ((NOT false), false), (NULL, (NOT (NOT true)))) AS L WHERE (((+(+(-((+110) / (+((-(-150)) * ((247 * (91 * (-47))) + (-86)))))))) = ((((+(+(24 / (+((+89) * (+58)))))) * (-(-((193 + 223) / (-(222 / 219)))))) * (34 * 70)) * (+(+((((+(+(-202))) / (+52)) - (-(228 + (-104)))) * (-24)))))) = (false <> (66 <> 8)));";
        let ast =
            parser::sqlparser_generate_ast(query).and_then(|it| Ok(fold_statement(it[0].clone())));
        assert_eq!(ast.unwrap(), None)
    }

    #[test]
    fn test_simple_math() {
        let query = "SELECT 2 + 3 * (4 - 1)";
        let ast =
            parser::sqlparser_generate_ast(query).and_then(|it| Ok(fold_statement(it[0].clone())));
        assert_eq!(ast.unwrap().unwrap().to_string(), "SELECT 11")
    }

    #[test]
    fn test_fold_query_with_insert() {
        let query = "INSERT INTO F SELECT * FROM (VALUES ((NOT false), false), (NULL, (NOT (NOT true)))) AS L WHERE (((+(+(-((+110) / (+((-(-150)) * ((247 * (91 * (-47))) + (-86)))))))) = ((((+(+(24 / (+((+89) * (+58)))))) * (-(-((193 + 223) / (-(222 / 219)))))) * (34 * 70)) * (+(+((((+(+(-202))) / (+52)) - (-(228 + (-104)))) * (-24)))))) = (false <> (66 <> 8)));";
        let ast =
            parser::sqlparser_generate_ast(query).and_then(|it| Ok(fold_statement(it[0].clone())));
        assert_eq!(
            ast.unwrap().unwrap().to_string(),
            "INSERT INTO F SELECT * FROM (VALUES (true, false), (NULL, true)) AS L WHERE false"
        )
    }
}
