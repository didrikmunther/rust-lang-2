use std::io::prelude::*;
use std::fs::File;
use ::std::io::{Write};

use lang::*;

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
    mode: Mode,
    code_offset: usize
}

impl<'a> Lang {
    pub fn new() -> Self {
        Lang {
            vm: vm::VM::new(),
            compiled: Vec::new(),
            mode: Mode::Run,
            code_offset: 0
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

                let lexed = lexer.lex(code.clone(), self.code_offset)?;
                let parsed = parser.parse(&lexed)?;
                let mut compiled = compiler.compile(&parsed)?;

                self.code_offset += code.len();

                self.compiled.append(&mut compiled);

                let executed = self.vm.exec(&self.compiled, offset)?;
                Ok(format!("{}", executed))
            },
            Mode::Lexed => {
                let lexed_res = lexer.lex(String::from(code.clone()), 0)
                    .map(|v| v.into_iter().map(|v| v.block_type).collect::<Vec<BlockType>>());
                Ok(format!("{:#?}", lexed_res))
            },
            Mode::Parsed => {
                let lexed = lexer.lex(code.clone(), 0)?;
                let parsed = parser.parse(&lexed)?;
                let parsed_res = parsed.into_iter().map(|v| v.declaration_type).collect::<Vec<DeclarationType>>();
                Ok(format!("{:#?}", parsed_res))
            },
            Mode::Compiled => {
                let lexed = lexer.lex(code.clone(), 0)?;
                let parsed = parser.parse(&lexed)?;
                let compiled = compiler.compile(&parsed)?;
                Ok(format!("{:#?}", compiled))
            }
        }
    }
}

fn shell() {
    let mut lang = Lang::new();
    let mut code = String::new();

    loop {
        print!("> ");
        flush();

        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).expect("Could not read user input.");

        code.push_str(&buf);

        match buf.as_ref() {
            "quit\n" => break,
            "$run\n" => lang.set_mode(Mode::Run),
            "$compiled\n" => lang.set_mode(Mode::Compiled),
            "$parsed\n" => lang.set_mode(Mode::Parsed),
            "$lexed\n" => lang.set_mode(Mode::Lexed),
            "$gc\n" => lang.vm.garbage(),
            _ => match lang.run(buf.as_ref()) {
                Ok(res) => println!("{}", res),
                Err(err) => println!("{}", err
                    .with_code(code.clone())
                    .with_file(String::from("[interactive shell]"))
                    // .with_file(String::from("src/main.lang"))
                )
            }
        };
    }
}

fn file(file_name: &str) {
    let mut lang = Lang::new();

    let mut file = File::open(file_name).expect("Unable to open the file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read the file");

    // lang.set_mode(Mode::Lexed);

    match lang.run(&contents) {
        Ok(res) => println!("{}", res),
        Err(err) => println!("{}", err
            .with_code(String::from(contents))
            .with_file(String::from(file_name))
            // .with_file(String::from("src/main.lang"))
        )
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        1 => shell(),
        2 => file(&args[1]),
        _ => println!("Wrong number of command line arguments")
    }
    
}