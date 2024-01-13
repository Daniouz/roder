use crate::parse::{Choice, OfType, Parser, Predicate, Repeatable, Sequence};

#[derive(Clone, PartialEq)]
pub enum TokenType {
    Semicolon,
    Dollar,
    Or,
    Caret,
    LBracket,
    RBracket,
    Equals,
    LParen,
    RParen,
    Eoi,
    Id(String),
    Str(String),
}

fn create_grammar_token_parser() -> impl Parser<TokenType> {
    let item = Sequence::from(
        "item",
        false,
        vec![
            Box::new(Predicate::from("id", false, |t| matches!(t, TokenType::Id(_)))),
            Box::new(OfType::from("=", false, TokenType::Equals)),
            Box::new(Predicate::from(
                "value",
                false,
                |t| matches!(t, TokenType::Str(_)),
            )),
        ]
    );

    Choice::from(
        "document",
        false,
        vec![
            Box::new(OfType::from("_", false, TokenType::Eoi)),
            Box::new(Sequence::from(
                "items",
                false,
                vec![
                    Box::new(Repeatable::from("fields", true, Box::new(item))),
                    Box::new(OfType::from("_", false, TokenType::Eoi)),
                ],
            )),
        ]
    )

    // TODO finish
}
