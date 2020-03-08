use std::collections::HashMap;

use Token::*;

type Definition<T> = HashMap<&'static str, T>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Token {
    EOF,
    SOF,
    Identifier,
    Literal,
    Rest,
    Comment,

    Space,
    Tab,
    NewLine,

    FSlash,
    Plus,
    Asterix,
    DoubleAsterix,
    Minus,
    Equals,
    DoubleEquals,
    MinusEquals,
    PlusEquals,
    ParClosed,
    ParOpen,
    BracketClosed,
    BracketOpen,
    BraceClosed,
    BraceOpen,
    SemiColon,
    Comma,
    Dot,
    Lambda
}

lazy_static! {
    pub static ref TOKENS: Definition<Token> = hashmap!{
        " " => Space,
        "\t" => Tab,
        "\n" => NewLine,

        "/" => FSlash,
        "+" => Plus,
        "*" => Asterix,
        "**" => DoubleAsterix,
        "-" => Minus,
        "=" => Equals,
        "==" => DoubleEquals,
        "-=" => MinusEquals,
        "+=" => PlusEquals,
        "(" => ParOpen,
        ")" => ParClosed,
        "{" => BracketOpen,
        "}" => BracketClosed,
        "[" => BraceOpen,
        "]" => BraceClosed,
        ";" => SemiColon,
        "," => Comma,
        "." => Dot,
        "=>" => Lambda
    };
}