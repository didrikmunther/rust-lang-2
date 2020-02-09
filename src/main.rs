use lang::*;

fn main() {
    let lexer = lexer::Lexer::new();

    // println!("{:#?}", lexer.lex(String::from("
    //     Hello, \"the//re \" handsome // this is a comment
    //     //2nd comment \"string 2\"
    // ")))

    println!("{:#?}", lexer.lex(String::from("
        a = 5 + 5 - 6 / 2 ** 1 ()
    ")));
}