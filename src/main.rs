use lang::*;

use ::std::io::{Write};

use lexer::BlockType;
use parser::DeclarationType;
use error::Error;

fn run(code: &str) -> Result<String, Error> {
    let lexer = lexer::Lexer::new();
    let mut parser = parser::Parser::new();

    let lexed = lexer.lex(String::from(code))?;
    let lexed_res = lexer.lex(String::from(code))?.into_iter().map(|v| v.block_type).collect::<Vec<BlockType>>();

    let parsed = parser.parse(&lexed)?;
    let parsed_res = parsed.into_iter().map(|v| v.declaration_type).collect::<Vec<DeclarationType>>();

    // Ok(String::from("Ok"))
    Ok(format!("{:?}\n{:#?}", lexed_res, parsed_res))
}

fn main() {  
    loop {
        print!("> ");
        std::io::stdout().flush().expect("Flush failed.");

        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).expect("Could not read user input.");

        match buf.as_ref() {
            "quit\n" => break,
            _ => match run(buf.as_ref()) {
                Ok(res) => println!("{}", res),
                Err(err) => println!("{}", err
                    .with_code(String::from(buf))
                    .with_file(String::from("src/main.lang"))
                )
            }
        };
    }
}