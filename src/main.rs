use lang::*;

use ::std::io;

use lexer::BlockType;
use parser::Parser;
use error::Error;

fn run(code: &str) -> Result<String, Error> {
    let lexer = lexer::Lexer::new();
    // let parser = parser::Parser::new();

    let lexed = lexer.lex(String::from(code))?;
    let lexed_res = lexed.into_iter().map(|v| v.block_type).collect::<Vec<BlockType>>();

    // let parsed = parser.parse

    // Ok(String::from("Ok"))
    Ok(format!("{:?}", lexed_res))
}

fn main() {  
    loop {
        print!("> ");

        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).expect("Could not read user input.");

        match run(buf.as_ref()) {
            Ok(res) => println!("{}", res),
            Err(err) => println!("{}", err
                .with_code(String::from(buf))
                .with_file(String::from("src/main.lang"))
            )
        };
    }
}