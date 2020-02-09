use std::fmt;

#[derive(Debug)]
pub enum LexerErrorType {
    UnexpectedEndOfString,
    UnknownToken
}

#[derive(Debug)]
pub enum ErrorType {
    LexerError(LexerErrorType)
}

#[derive(Debug)]
pub struct Error {
    code: Option<String>,
    file: Option<String>,
    help: Option<String>,
    pub pos: usize,
    pub width: usize,
    pub error_type: ErrorType
}

impl Error {
    pub fn new(pos: usize, width: usize, error_type: ErrorType) -> Self {
        Self {
            code: None,
            file: None,
            help: None,
            pos,
            width,
            error_type
        }
    }

    pub fn with_code(mut self, code: String) -> Self {
        self.code = Some(code);
        return self;
    }

    pub fn with_file(mut self, file: String) -> Self {
        self.file = Some(file);
        return self;
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help);
        return self;
    }
}

fn repeat(sequence: &'static str, amount: usize) -> String {
    let mut res = String::new();

    for _ in 0..amount {
        res.push_str(sequence);
    }

    res
}

fn get_line_pos(code: &str, pos: usize) -> (usize, usize, usize, usize) {
    let mut line_pos = 0;
    let mut newlines = 0;
    let mut indents = 0;
    let mut line_indents = 0;

    let chars = code.as_bytes();
    let len = code.len();

    for i in 0..len {
        if chars[i] == '\n' as u8 {
            if i >= pos {
                break;
            }

            newlines += 1;
            indents = 0;
            line_indents = 0;
            line_pos = i + 1;
        } else {
            line_indents += 1;
            if i < pos {
                indents += 1;
            }
        }
    }

    (line_pos, newlines, indents, line_indents)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let None = self.code {
            return write!(f, "No code supplied for error")
        }

        let code = self.code.as_ref().unwrap();
        let (line_pos, line, indents, line_indents) = get_line_pos(code, self.pos);

        write!(
            f,
            "error: {:?}\n  --> {}{}:{}\n   | {}\n   | {}{} {}",
            self.error_type,
            if let Some(file) = &self.file { format!("{}:", file) } else { String::from("") },
            line,
            indents,
            &code[line_pos..line_pos + line_indents],
            repeat("-", indents),
            repeat("^", self.width),
            if let Some(help) = &self.help { format!("tip: {}", help) } else { String::from("") }
        )
    }
}