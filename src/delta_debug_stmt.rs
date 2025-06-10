use crate::driver::test_query;
use crate::reducer::remove_tables_in_place;
use crate::statements::types::Statement;
use crate::utils::vec_statement_to_string;
use log::*;
use std::collections::HashSet;
use std::error::Error;

/// Split `tests` into `n` parts as evenly as possible.
fn split_tests<T: Clone>(tests: &[T], n: usize) -> Vec<Vec<T>> {
    let mut parts: Vec<Vec<T>> = Vec::with_capacity(n);
    let len = tests.len();
    let rem = len % n;
    let base = len / n;
    let mut start = 0;

    for i in 0..n {
        let end = start + base + if i < rem { 1 } else { 0 };
        parts.push(tests[start..end].to_vec());
        start = end;
    }

    parts
}

/// Returns the elements in `haystack` that are *not* in `needles`.
fn difference<T: Eq + std::hash::Hash + Clone>(haystack: &[T], needles: &[T]) -> Vec<T> {
    let forbidden: HashSet<_> = needles.iter().cloned().collect();
    haystack
        .iter()
        .filter(|item| !forbidden.contains(*item))
        .cloned()
        .collect()
}

/// Perform "delta-inclusion" debugging on a vector of items of arbitrary type T.
/// Finds a (greedy) maximal subset of `data` for which `test_query` still succeeds.
pub fn delta_debug_stmt<T>(
    data: Vec<T>,
    mut granularity: usize,
    query: &Vec<Statement>,
) -> Result<Vec<T>, Box<dyn Error>>
where
    T: Clone + AsRef<str> + PartialEq + std::fmt::Display + Eq + std::hash::Hash,
{
    let mut base: Vec<T> = Vec::new();
    let mut remaining = data;

    while !remaining.is_empty() && granularity <= remaining.len() {
        let tests = split_tests(&remaining, granularity);
        let mut progressed = false;

        for chunk in &tests {
            let mut trial = base.clone();
            trial.extend(chunk.iter().cloned());
            let tmp = remove_tables_in_place(&trial, query);
            let input = vec_statement_to_string(&tmp, ";")?;

            if test_query(&input)? {
                base = trial;
                remaining = difference(&remaining, chunk);
                granularity = granularity.saturating_mul(2);
                progressed = true;
                break;
            }

            let complement = difference(&remaining, chunk);
            let mut trial2 = base.clone();
            trial2.extend(complement.iter().cloned());
            let tmp2 = remove_tables_in_place(&trial2, query);
            let input2 = vec_statement_to_string(&tmp2, ";")?;

            if test_query(&input2)? {
                remaining = complement;
                granularity = granularity.saturating_mul(2);
                progressed = true;
                break;
            }
        }

        if !progressed {
            granularity = granularity.saturating_add(1);
        }
    }

    for e in remaining {
        let mut trial = base.clone();
        trial.push(e.clone());
        let tmp = remove_tables_in_place(&trial, query);
        let input = vec_statement_to_string(&tmp, ";")?;
        if test_query(&input)? {
            base.push(e);
        }
    }

    Ok(base)
}

/// Returns all items in `data` that are *not* in `delta`.
fn get_nabla<T: Clone + PartialEq>(data: &[T], delta: &[T]) -> Vec<T> {
    data.iter()
        .filter(|item| !delta.contains(item))
        .cloned()
        .collect()
}

#[test]
fn test_split_tests() {
    let tests = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let parts = split_tests(&tests, 3);
    assert_eq!(parts, vec![vec![1, 2, 3, 4], vec![5, 6, 7], vec![8, 9, 10]]);
}

#[test]
fn test_get_nabla() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let delta = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    let nabla = get_nabla(&data, &delta);
    assert_eq!(nabla, vec![10]);
}
