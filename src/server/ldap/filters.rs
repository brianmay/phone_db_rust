/// Escape an LDAP filter value according to RFC 4515.
/// This prevents LDAP injection by escaping characters that have
/// special meaning in filter expressions.
pub fn escape_filter_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for c in value.chars() {
        match c {
            '*' => escaped.push_str(r"\2a"),
            '(' => escaped.push_str(r"\28"),
            ')' => escaped.push_str(r"\29"),
            '\\' => escaped.push_str(r"\5c"),
            '\0' => escaped.push_str(r"\00"),
            _ => escaped.push(c),
        }
    }

    escaped
}

/// Escape a DN attribute value according to RFC 4514.
/// DN escaping rules are different from filter escaping.
pub fn escape_dn_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for (i, c) in value.chars().enumerate() {
        match c {
            ',' => escaped.push_str(r"\,"),
            '+' => escaped.push_str(r"\+"),
            '"' => escaped.push_str(r#"\""#),
            '\\' => escaped.push_str(r"\\"),
            '<' => escaped.push_str(r"\<"),
            '>' => escaped.push_str(r"\>"),
            ';' => escaped.push_str(r"\;"),
            // Leading or trailing space must be escaped
            ' ' if i == 0 || i == value.len() - 1 => escaped.push_str(r"\ "),
            '#' if i == 0 => escaped.push_str(r"\#"),
            _ => escaped.push(c),
        }
    }

    escaped
}

/// Build a safe equality filter: (attr=value)
pub fn eq(attr: &str, value: &str) -> String {
    format!("({}={})", attr, escape_filter_value(value))
}

/// Build a safe AND filter: (&(a=1)(b=2))
#[allow(dead_code)]
pub fn and(filters: &[String]) -> String {
    let inner = filters.join("");
    format!("(&{})", inner)
}

/// Build a safe OR filter: (|(a=1)(b=2))
#[allow(dead_code)]
pub fn or(filters: &[String]) -> String {
    let inner = filters.join("");
    format!("(|{})", inner)
}
