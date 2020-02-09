use lang::*;

fn main() {
    let lexer = lexer::Lexer::new();

    // println!("{:#?}", lexer.lex(String::from("
    //     Hello, \"the//re \" handsome // this is a comment
    //     //2nd comment \"string 2\"
    // ")))

    let code = "
        a = 5 + 5 - 6 / 2 ** 1 (  )
    ";
    // let code = "Hello, \"there   \"       asdf  \"   a 
    //     asdf
    // ";

    let res = lexer.lex(String::from(code));

    match res {
        Ok(res) => println!("{:#?}", res),
        Err(err) => println!("{}", err
            .with_code(String::from(code))
            .with_file(String::from("src/main.lang"))
        )
    };
}