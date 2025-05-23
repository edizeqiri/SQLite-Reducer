// src/transform/constant_fold.rs

use crate::transformation::transformer::Transform;
use sqlparser::ast::Value::{Boolean, Number};
use sqlparser::ast::{
    BinaryOperator, Expr as SQLExpr, Expr, Expr as Value, Query, SelectItem, SetExpr, Statement,
    UnaryOperator, ValueWithSpan,
};

/// Our pass struct — no state needed for basic constant folding
#[derive(Debug, Default)]
pub struct ConstantFold;

impl Transform for ConstantFold {
    fn apply(&self, stmt: Statement) -> Statement {
        fold_statement(stmt).unwrap_or_else(|| {
            // Create a minimal query that returns no rows
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
        })
    }
}

/// Top-level dispatch
fn fold_statement(stmt: Statement) -> Option<Statement> {
    match stmt {
        Statement::Query(boxed_q) => {
            let q = *boxed_q;
            let new_body = fold_setexpr(*q.body);

            if let SetExpr::Select(select) = &new_body {
                if let Some(SQLExpr::Value(ValueWithSpan {
                    value: Boolean(false),
                    ..
                })) = select.selection
                {
                    return None;
                }
            }

            Some(Statement::Query(Box::new(Query {
                body: Box::new(new_body),
                ..q
            })))
        }
        // pass through everything else unchanged
        other => Some(other),
    }
}

fn fold_setexpr(setexpr: SetExpr) -> SetExpr {
    match setexpr {
        SetExpr::Select(mut select) => {
            select.projection = select
                .projection
                .into_iter()
                .map(|item| match item {
                    SelectItem::UnnamedExpr(expr) => SelectItem::UnnamedExpr(fold_expr(expr)),
                    other => other,
                })
                .collect();

            select.from = select
                .from
                .into_iter()
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
                .collect();

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

/// Recursively fold an expression
fn fold_expr(expr: SQLExpr) -> SQLExpr {
    use SQLExpr::*;
    match expr {
        // Handle NOT expressions first
        UnaryOp {
            op,
            expr: boxed_expr,
        } => {
            let inner = fold_expr(*boxed_expr);
            match op {
                UnaryOperator::Not => match inner {
                    Value(ValueWithSpan {
                        value: Boolean(b), ..
                    }) => Value(Boolean(!b).with_empty_span()),
                    UnaryOp {
                        op: UnaryOperator::Not,
                        expr: inner_boxed,
                    } => fold_expr(*inner_boxed),
                    _ => UnaryOp {
                        op,
                        expr: Box::new(inner),
                    },
                },
                UnaryOperator::Plus => inner,
                UnaryOperator::Minus => {
                    if let Value(ValueWithSpan {
                        value: Number(n, _),
                        ..
                    }) = inner
                    {
                        if let Ok(num) = n.parse::<f64>() {
                            let result = -num;
                            if result.fract() == 0.0 {
                                Value(
                                    Number(format!("{}", result.trunc() as i64), false)
                                        .with_empty_span(),
                                )
                            } else {
                                Value(Number(format!("{}", result), false).with_empty_span())
                            }
                        } else {
                            UnaryOp {
                                op,
                                expr: Box::new(Value(ValueWithSpan {
                                    value: Number(n.clone(), false),
                                    span: sqlparser::tokenizer::Span::empty(),
                                })),
                            }
                        }
                    } else {
                        UnaryOp {
                            op,
                            expr: Box::new(inner),
                        }
                    }
                }
                _ => UnaryOp {
                    op,
                    expr: Box::new(inner),
                },
            }
        }
        // Handle nested expressions
        Nested(inner) => {
            let folded = fold_expr(*inner);
            if let Value(_) = folded {
                folded
            } else {
                Nested(Box::new(folded))
            }
        }
        // Handle binary operations
        BinaryOp { left, op, right } => {
            let l = fold_expr(*left);
            let r = fold_expr(*right);

            // Handle boolean operations
            if let (
                Value(ValueWithSpan {
                    value: Boolean(lb), ..
                }),
                Value(ValueWithSpan {
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
                        return BinaryOp {
                            left: Box::new(l),
                            op,
                            right: Box::new(r),
                        }
                    }
                };
                return Value(Boolean(bool_result).with_empty_span());
            }

            // Handle numeric operations
            if let (
                Value(ValueWithSpan {
                    value: Number(ls, _),
                    ..
                }),
                Value(ValueWithSpan {
                    value: Number(rs, _),
                    ..
                }),
            ) = (&l, &r)
            {
                if let (Ok(ln), Ok(rn)) = (ls.parse::<f64>(), rs.parse::<f64>()) {
                    match op {
                        BinaryOperator::Plus => {
                            let result = ln + rn;
                            return if result.fract() == 0.0 {
                                Value(
                                    Number(format!("{}", result.trunc() as i64), false)
                                        .with_empty_span(),
                                )
                            } else {
                                Value(Number(format!("{}", result), false).with_empty_span())
                            };
                        }
                        BinaryOperator::Minus => {
                            let result = ln - rn;
                            return if result.fract() == 0.0 {
                                Value(
                                    Number(format!("{}", result.trunc() as i64), false)
                                        .with_empty_span(),
                                )
                            } else {
                                Value(Number(format!("{}", result), false).with_empty_span())
                            };
                        }
                        BinaryOperator::Multiply => {
                            let result = ln * rn;
                            return if result.fract() == 0.0 {
                                Value(
                                    Number(format!("{}", result.trunc() as i64), false)
                                        .with_empty_span(),
                                )
                            } else {
                                Value(Number(format!("{}", result), false).with_empty_span())
                            };
                        }
                        BinaryOperator::Divide => {
                            let result = ln / rn;
                            return if result.fract() == 0.0 {
                                Value(
                                    Number(format!("{}", result.trunc() as i64), false)
                                        .with_empty_span(),
                                )
                            } else {
                                Value(Number(format!("{}", result), false).with_empty_span())
                            };
                        }
                        BinaryOperator::Eq => return Value(Boolean(ln == rn).with_empty_span()),
                        BinaryOperator::NotEq => return Value(Boolean(ln != rn).with_empty_span()),
                        BinaryOperator::Gt => return Value(Boolean(ln > rn).with_empty_span()),
                        BinaryOperator::Lt => return Value(Boolean(ln < rn).with_empty_span()),
                        BinaryOperator::GtEq => return Value(Boolean(ln >= rn).with_empty_span()),
                        BinaryOperator::LtEq => return Value(Boolean(ln <= rn).with_empty_span()),
                        _ => {}
                    }
                }
            }

            // Otherwise, reconstruct
            BinaryOp {
                left: Box::new(l),
                op,
                right: Box::new(r),
            }
        }
        // Pass through other expressions unchanged
        other => other,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser;

    #[test]
    fn test_fold_query1() {
        let query = "SELECT * FROM (VALUES ((NOT false), false), (NULL, (NOT (NOT true)))) AS L WHERE (((+(+(-((+110) / (+((-(-150)) * ((247 * (91 * (-47))) + (-86)))))))) = ((((+(+(24 / (+((+89) * (+58)))))) * (-(-((193 + 223) / (-(222 / 219)))))) * (34 * 70)) * (+(+((((+(+(-202))) / (+52)) - (-(228 + (-104)))) * (-24)))))) = (false <> (66 <> 8)));";
        let ast = parser::generate_ast(query).and_then(|it| Ok(fold_statement(it[0].clone())));
        assert_eq!(ast.unwrap(), None)
    }

    #[test]
    fn test_simple_math() {
        let query = "SELECT 2 + 3 * (4 - 1)";
        let ast = parser::generate_ast(query).and_then(|it| Ok(fold_statement(it[0].clone())));
        assert_eq!(ast.unwrap().unwrap().to_string(), "SELECT 11")
    }
}
