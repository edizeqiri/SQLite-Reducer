use crate::transformation::constant_fold::ConstantFold;
use sqlparser::ast::Statement;

pub trait Transform {
    fn apply(&self, stmt: Statement) -> Statement;
}

pub enum TransformPass {
    ConstantFold(ConstantFold),
    /*PredicatePushdown(PredicatePushdown),
    ProjectionPrune(ProjectionPrune),
    FlattenSubqueries(FlattenSubqueries),
    JoinReorder(JoinReorder),*/
}

impl Transform for TransformPass {
    fn apply(&self, stmt: Statement) -> Statement {
        match self {
            TransformPass::ConstantFold(tf) => tf.apply(stmt),
            /*TransformPass::PredicatePushdown(tf)  => tf.apply(stmt),
            TransformPass::ProjectionPrune(tf)    => tf.apply(stmt),
            TransformPass::FlattenSubqueries(tf)  => tf.apply(stmt),
            TransformPass::JoinReorder(tf)        => tf.apply(stmt),*/
        }
    }
}

pub fn transform(mut stmts: Vec<Statement>) -> Vec<Statement> {
    let transforms: Vec<TransformPass> = vec![
        TransformPass::ConstantFold(ConstantFold {}),
        /*TransformPass::PredicatePushdown(PredicatePushdown{}),
        TransformPass::ProjectionPrune(ProjectionPrune{}),
        TransformPass::FlattenSubqueries(FlattenSubqueries{}),
        TransformPass::JoinReorder(JoinReorder{}),*/
    ];

    for pass in &transforms {
        stmts = stmts
            .clone()
            .into_iter()
            .map(|stmt| pass.apply(stmt))
            .collect();
    }
    stmts
}
