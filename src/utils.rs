pub fn vec_statement_to_string<T>(vector: &Vec<T>, separator: &str) -> String
where
    T: ToString + PartialEq,
{
    vector
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(separator)
        + separator
}
