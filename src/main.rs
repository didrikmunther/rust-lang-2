use lang::*;

use ::std::io::{Write};

use lexer::BlockType;
use parser::DeclarationType;
use error::Error;

struct Lang {
    vm: vm::VM,
    is_debug: bool
}

impl<'a> Lang {
    pub fn new() -> Self {
        Lang {
            vm: vm::VM::new(),
            is_debug: false
        }
    }

    pub fn debug(&mut self, debug: bool) -> &mut Self {
        self.is_debug = debug;
        self
    }

    pub fn run(&mut self, code: &str) -> Result<String, Error> {
        let lexer = lexer::Lexer::new();
        let mut parser = parser::Parser::new();
        let mut compiler = compiler::Compiler::new();

        let code = String::from(code);

        let lexed = lexer.lex(code.clone())?;
        let parsed = parser.parse(&lexed)?;
        let compiled = compiler.compile(&parsed)?;
        let executed = self.vm.exec(&compiled)?;
    
        if self.is_debug {
            let lexed_res = lexer.lex(String::from(code.clone()))?.into_iter().map(|v| v.block_type).collect::<Vec<BlockType>>();
            let parsed_res = parser.parse(&lexed)?.into_iter().map(|v| v.declaration_type).collect::<Vec<DeclarationType>>();
            Ok(format!("{:#?}\n{:#?}\n{:#?}\n{}", lexed_res, parsed_res, compiled, executed))
        } else {
            Ok(format!("{}", executed))
        }
    }
}

fn main() {
    let mut lang = Lang::new();
    let mut debug = false;

    loop {
        print!("> ");
        std::io::stdout().flush().expect("Flush failed.");

        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).expect("Could not read user input.");

        match buf.as_ref() {
            "quit\n" => break,
            "debug\n" => {
                debug = !debug;
                println!("Debug is [{}]", debug);
            },
            _ => match lang.debug(debug).run(buf.as_ref()) {
                Ok(res) => println!("{}", res),
                Err(err) => println!("{}", err
                    .with_code(String::from(buf))
                    .with_file(String::from("[interactive shell]"))
                    // .with_file(String::from("src/main.lang"))
                )
            }
        };
    }
}