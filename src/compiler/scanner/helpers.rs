pub const INJUNCTIONS: &'static [&'static str; 13] = &[
    "implement",
    "interface",
    "struct",
    "public",
    "utilize",
    "record",
    "const",
    "let",
    "enum",
    "function",
    "type",
    "use",
    "prepend",
];

pub const KEYWORDS: &'static [&'static str; 22] = &[
    "if", "else", "for", "fn", "in", "of", "match", "loop", "case", "from", "as", "do", "while",
    "break", "continue", "return", "crash", "try", "recover", "println", "readonly", "static",
];

pub const OPERATORS: &'static [&'static str; 38] = &[
    "&&=", "||=", "...", "||", "&&", "..", ">>", "<<", "=>", "->", ">=", "<=", "!=", "==", "*=",
    "-=", "/=", "%=", "+=", "++", "**", "--", "::", "^", ".", "=", "+", "-", ">", "<", "/", "&",
    "|", "%", "*", "!", "~", "?",
];

pub const LITERALS: &'static [&'static str; 6] = &["true", "false", "self", "core", "nil", "super"];

/// Return the precedence of an operator according to the Pebble precedence chart.
/// An operator with a higher precedence has a higher influence on parsing arrangement than one with a lower precedence.
pub fn precedence_of(operator: &str) -> usize {
    match operator {
        "." => 13,
        "[" => 12,
        "--" => 11,
        "++" => 11,
        "**" => 10,
        "*" => 9,
        "+" => 8,
        "-" => 8,
        "&&" => 5,
        "||" => 4,
        ".." => 3,
        "=" => 2,
        _ => 0,
    }
}

pub fn is_identifier_char(c: char) -> bool {
    c == '$' || c == '_' || c.is_alphabetic() || c.is_digit(10)
}
pub fn is_injunction_value(value: &str) -> bool {
    INJUNCTIONS.contains(&value)
}
pub fn is_keyword(value: &str) -> bool {
    KEYWORDS.contains(&value)
}
pub fn is_literal(value: &str) -> bool {
    LITERALS.contains(&value)
}
