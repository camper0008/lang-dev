pub fn unescape_string(source: &str) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = source.chars();
    loop {
        match chars.next() {
            Some('\\') => match chars.next() {
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('n') => result.push('\n'),
                Some('0') => result.push('\0'),
                Some(char) => result.push(char),
                None => {
                    break Err("malformed escape sequence, expected char after '\\'".to_string())
                }
            },
            Some(char) => result.push(char),
            None => break Ok(result),
        }
    }
}

pub fn escape_string(source: &str) -> String {
    let mut result = String::new();
    for c in source.chars() {
        match c {
            '\"' => result.push_str("\\\""),
            '\'' => result.push_str("\\\'"),
            '\0' => result.push_str("\\0"),
            '\\' => result.push_str("\\\\"),
            char => result.push(char),
        }
    }
    result
}
