use crate::driver::test_query;
use std::error::Error;


/// Perform delta debugging on a vector of items of arbitrary type T.
pub fn delta_debug<T>(mut data: Vec<T>, mut granularity: usize) -> Result<Vec<T>, Box<dyn Error>>
where
    T: Clone + ToString + std::cmp::PartialEq,
{
    while granularity <= data.len() {
        let tests = split_tests(&data, granularity);
        let mut reduced = false;

        for delta in &tests {
            let nabla = get_nabla(&data, delta);

            let input_delta = delta
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(";") + ";";
            let input_nabla = nabla
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(";") + ";";

            // use `?` to propagate any I/O/test errors
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

    find_one_minimal(&data)
}

/// Recursively remove one element at a time.
fn find_one_minimal<T>(test: &[T]) -> Result<Vec<T>, Box<dyn Error>>
where
    T: Clone + ToString,
{
    let mut current = test.to_vec();
    for i in 0..current.len() {
        let mut truncated = current.clone();
        truncated.remove(i);

        let input = truncated
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(";") + ";";
        if test_query(&input)? {
            return find_one_minimal(&truncated);
        }
    }
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

