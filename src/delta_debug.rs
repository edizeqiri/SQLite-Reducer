use std::path::PathBuf;
use sqlparser::ast::Statement;
use crate::driver::{test_query, Setup};


pub fn ddmin(mut stmts: Vec<Statement>, setup: Setup) -> Vec<Statement> {
    /// Try removing one of `n` chunks from `stmts`.  If removing chunk `i` still fails,
    /// recurse immediately on that smaller vector (with granularity reset to 2).
    fn reduce(stmts: Vec<Statement>, setup: Setup, n: usize) -> Vec<Statement> {
        let len = stmts.len();
        // If only one stmt left, that's as small as we get.
        if len <= 1 {
            return stmts;
        }

        let chunk_size = (len + n - 1) / n; // ceil
        let mut start = 0;
        for _ in 0..n {
            let end = (start + chunk_size).min(len);
            // build candidate by skipping [start..end)
            let mut candidate = Vec::with_capacity(len - (end - start));
            candidate.extend_from_slice(&stmts[0..start]);
            candidate.extend_from_slice(&stmts[end..]);

            // re-render back to SQL
            let sql = candidate
                .iter()
                .map(|stmt| stmt.to_string())
                .collect::<Vec<_>>()
                .join("\n");

            // if the bug still reproduces, recurse!
            if test_query(setup.clone(), sql).expect("TODO: handle error") {
                return reduce(candidate, setup, 2);
            }

            start = end;
            if start >= len {
                break;
            }
        }

        // no single chunk worked; try a finer split (up to len)
        if n < len {
            reduce(stmts, setup, (n * 2).min(len))
        } else {
            // we can’t split any further, so this is minimal
            stmts
        }
    }

    // kick off with binary granularity
    reduce(stmts, setup, 2)
}
