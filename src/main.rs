use lang::*;

use ::std::io::{Write};

use lexer::BlockType;
use parser::DeclarationType;
use error::Error;
use compiler::Program;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum Mode {
    Run = 0,
    Compiled = 1,
    Parsed = 2,
    Lexed = 3
}

fn flush() {
    std::io::stdout().flush().expect("Flush failed.");
}

struct Lang {
    vm: vm::VM,
    compiled: Program,
    mode: Mode
}

impl<'a> Lang {
    pub fn new() -> Self {
        Lang {
            vm: vm::VM::new(),
            compiled: Vec::new(),
            mode: Mode::Run
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        if mode != self.mode {
            println!("Mode switched to [{:?}]", mode);
            flush();
        }

        self.mode = mode;
    }

    pub fn run(&mut self, code: &str) -> Result<String, Error> {
        let lexer = lexer::Lexer::new();
        let mut parser = parser::Parser::new();
        let mut compiler = compiler::Compiler::new();

        let code = String::from(code);

        match self.mode {
            Mode::Run => {
                let offset = self.compiled.len();

                let lexed = lexer.lex(code.clone())?;
                let parsed = parser.parse(&lexed)?;
                let mut compiled = compiler.compile(&parsed)?;

                self.compiled.append(&mut compiled);

                let executed = self.vm.exec(&self.compiled, offset)?;
                Ok(format!("{}", executed))
            },
            Mode::Lexed => {
                let lexed_res = lexer.lex(String::from(code.clone()))
                    .map(|v| v.into_iter().map(|v| v.block_type).collect::<Vec<BlockType>>());
                Ok(format!("{:#?}", lexed_res))
            },
            Mode::Parsed => {
                let lexed = lexer.lex(code.clone())?;
                let parsed = parser.parse(&lexed)?;
                let parsed_res = parsed.into_iter().map(|v| v.declaration_type).collect::<Vec<DeclarationType>>();
                Ok(format!("{:#?}", parsed_res))
            },
            Mode::Compiled => {
                let lexed = lexer.lex(code.clone())?;
                let parsed = parser.parse(&lexed)?;
                let compiled = compiler.compile(&parsed)?;
                Ok(format!("{:#?}", compiled))
            }
        }
    }
}

fn main() {
    let mut lang = Lang::new();

    loop {
        print!("> ");
        flush();

        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).expect("Could not read user input.");

        match buf.as_ref() {
            "quit\n" => break,
            "$run\n" => lang.set_mode(Mode::Run),
            "$compiled\n" => lang.set_mode(Mode::Compiled),
            "$parsed\n" => lang.set_mode(Mode::Parsed),
            "$lexed\n" => lang.set_mode(Mode::Lexed),
            _ => match lang.run(buf.as_ref()) {
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