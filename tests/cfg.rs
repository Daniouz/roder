#[derive(Clone)]
enum TokenType {
    Identifier(String),
    Equals,
    String(String),
    Semicolon,
    Eoi,
}

const GRAMMAR: &str = r#"

"#;

const SRC: &str = r#"
name = "Gary";
age = 78;
"#;
