use crate::driver::test_query;
use crate::statements::types::Statement;
use crate::utils::vec_statement_to_string;
use log::*;
use std::error::Error;

/// Perform delta debugging on a vector of items of arbitrary type T.
pub fn bruteforce_delta_debug(
    mut data: Vec<String>,
    mut granularity: usize,
    index: usize,
    original_query: &Vec<String>
) -> Result<String, Box<dyn Error>> {

    while granularity > 1 && granularity <= data.len() {
        let tests = split_tests(&data, granularity);
        let mut reduced = false;

        for mut delta in tests {

            let mut orig_delta = original_query.clone();
            let mut orig_nabla = original_query.clone();
            let mut nabla = get_nabla(&data, &delta);

            delta = string_w_para(&delta);
            nabla = string_w_para(&nabla);

            if (delta.len() == data.len() || nabla.len() == data.len()) {
                break;
            };

            let mut input_delta = delta.join(" ");
            if !input_delta.ends_with(';') {
                input_delta.push(';'); 
            }

            let mut input_nabla = nabla.join(" ");
            if !input_nabla.ends_with(';') {
                input_nabla.push(';');
            }

            info!("delta: {}", input_delta);
            info!("nabla: {}", input_nabla);

            orig_delta[index] = input_delta;
            orig_nabla[index] = input_nabla;

            input_delta = vec_statement_to_string(&orig_delta, ";")?;
            input_nabla = vec_statement_to_string(&orig_nabla, ";")?;

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

pub fn find_one_minimal(statements: &[String]) -> Result<String, Box<dyn Error>> {
    let current = statements.to_vec();
    for i in 0..current.len() {
        let mut truncated = current.clone();
        truncated.remove(i);

        let query = string_w_para(&truncated);

        let mut input = query.join(" ");
        if !input.ends_with(';') {
            input.push(';');
        }

        if test_query(&input)? {
            return find_one_minimal(&truncated);
        }
    }

    let query = string_w_para(&current);

    let mut input = query.join(" ");
    if !input.ends_with(';') {
        input.push(';');
    }

    info!("Minimal sequence: {:?}", input);
    Ok(input)
}

/// Splits `tests` into `n` parts as evenly as possible.
pub fn split_tests(tests: &[String], n: usize) -> Vec<Vec<String>> {
    let len = tests.len();
    let rem = len % n;
    let base = len / n;
    let mut parts: Vec<Vec<String>> = Vec::with_capacity(n);
    let mut start = 0;

    for i in 0..n {
        let end = start + base + if i < rem { 1 } else { 0 };
        parts.push(tests[start..end].to_vec());
        start = end;
    }

    parts
}

/// Returns all items in `data` that are *not* in `delta`.
pub fn get_nabla(data: &[String], delta: &[String]) -> Vec<String> {
    data.iter()
        .filter(|item| !delta.contains(item))
        .cloned()
        .collect()
}


fn string_w_para(queries : &Vec<String>) -> Vec<String> {
    let mut quer = queries.clone();
    if find_first_paren(&queries) == Some(')') {
        quer.insert(0, '('.to_string());
    };
    if find_last_paren(queries) == Some('(') {
        quer.push(')'.to_string());
    };
    quer
}

fn find_first_paren(items: &[String]) -> Option<char> {
    items
        .iter()
        .find_map(|s| s.chars().find(|&c| c == '(' || c == ')'))
}

/// Returns the first '(' or ')' found when scanning **backward** through
/// all of `items` (i.e. starting at the end), or `None` if there isn't one.
fn find_last_paren(items: &[String]) -> Option<char> {
    items
        .iter()
        .rev()
        .find_map(|s| s.chars().rev().find(|&c| c == '(' || c == ')'))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_tests() {
        let tests = vec![
            "a".into(), "b".into(), "c".into(), "d".into(), "e".into(),
        ];
        let parts = split_tests(&tests, 3);
        assert_eq!(parts, vec![
            vec!["a", "b"],
            vec!["c", "d"],
            vec!["e"],
        ]);
    }

    #[test]
    fn test_get_nabla() {
        let data = vec!["1".into(), "2".into(), "3".into()];
        let delta = vec!["1".into(), "2".into()];
        let nabla = get_nabla(&data, &delta);
        assert_eq!(nabla, vec!["3"]);
    }
}