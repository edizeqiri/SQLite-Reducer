use crate::driver::test_query;
use crate::utils::vec_statement_to_string;
use log::*;
use std::error::Error;

/// Perform delta debugging on a vector of items of arbitrary type T.
pub fn delta_debug<T>(mut data: Vec<T>, mut granularity: usize) -> Result<Vec<T>, Box<dyn Error>>
where
    T: Clone + ToString + std::cmp::PartialEq + std::fmt::Debug,
{
    while granularity > 1 && granularity <= data.len() {
        let tests = split_tests(&data, granularity);
        let mut reduced = false;

        for delta in &tests {
            let nabla = get_nabla(&data, delta);

            let input_delta = vec_statement_to_string(&delta, ";")?;
            let input_nabla = vec_statement_to_string(&nabla, ";")?;

            info!("delta: {}", delta.len());
            info!("nabla: {}", nabla.len());

            if test_query(&input_delta)? {
                data = delta.clone();
                reduced = true;
                break;
            } else if test_query(&input_nabla)? {
                data = nabla;
                granularity = granularity.saturating_sub(1);
                reduced = true;
                break;
            }
        }

        if !reduced {
            granularity = granularity.saturating_mul(2);
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
