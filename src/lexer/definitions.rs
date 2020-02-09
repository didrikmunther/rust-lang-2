use std::collections::HashMap;

use Token::*;

type Definition<T> = HashMap<&'static str, T>;

#[derive(Copy, Clone, Debug)]
pub enum Token {
    Space,
    Tab,
    NewLine,

    FSlash,
    Plus,
    Asterix,
    Minus,
    Equals,
}

lazy_static! {
    pub static ref TOKENS: Definition<Token> = hashmap!{
        " " => Space,
        "\t" => Tab,
        "\n" => NewLine,

        "/" => FSlash,
        "+" => Plus,
        "*" => Asterix,
        "-" => Minus,
        "=" => Equals
    };
}