use crate::driver::test_query;
use crate::reducer::remove_tables_in_place;
use crate::statements::types::Statement;
use crate::utils::vec_statement_to_string;
use log::*;
use std::error::Error;

/// Perform delta debugging on a vector of items of arbitrary type T.
/// IN THIS FILE: only statements are reduced
/// We want to find the list of tables (here data) that can be removed (=> Return nabla of table)
pub fn delta_debug_stmt<T>(mut data: Vec<T>, mut granularity: usize, query: &Vec<Statement>) -> Result<Vec<T>, Box<dyn Error>>
where
    T: Clone + AsRef<str> + PartialEq + std::fmt::Display,
{
    if data.len()== 1 && granularity > 1 { granularity = 1 }
    while granularity > 0 && granularity <= data.len() {
        let tests = split_tests(&data, granularity);
        let mut reduced = false;

        for delta in &tests {
            let nabla = get_nabla(&data, delta);

            // remove the chosen table and its opposite
            let tmp_delta = remove_tables_in_place(&delta, query.to_vec());
            let tmp_nabla = remove_tables_in_place(&nabla, query.to_vec());

            let input_delta = vec_statement_to_string(&tmp_delta, ";")?;
            let input_nabla = vec_statement_to_string(&tmp_nabla, ";")?;

            info!("delta: {}", delta.len());
            info!("nabla: {}", nabla.len());

            // use `?` to propagate any I/O/test errors
            if test_query(&input_delta)? {
                data = delta.clone();
                reduced = true;
                break;
            } else if test_query(&input_nabla)? {
                data = nabla;
                granularity = granularity.saturating_mul(2);
                reduced = true;
                break;
            }
        }

        if !reduced {
            granularity = granularity.saturating_sub(1);
        }
    }

    let minimal_statement = find_one_minimal(&data);

    minimal_statement
}



/// Recursively remove one element at a time.
fn find_one_minimal<T>(test: &[T]) -> Result<Vec<T>, Box<dyn Error>>
where
    T: Clone + ToString + PartialEq,
{
    let current = test.to_vec();
    for i in 0..current.len() {
        let mut truncated = current.clone();
        truncated.remove(i);

        let input = vec_statement_to_string(&truncated, ";");
        if test_query(&input?)? {
            return find_one_minimal(&truncated);
        }
    }

    info!("{:?}", vec_statement_to_string(&current, ";"));
    Ok(current)
}

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
