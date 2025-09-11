use crate::token::Token;
//Lexer Struct
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}
//Lexer Constructor
impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }
    //Peeking and Advancing through the code
    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).cloned()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    //Skipping WhiteSpaces
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    //Main Tokenization Function
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let ch = match self.advance() {
            Some(c) => c,
            None => return Token::EOF,
        };
        //Matching Signs
        match ch {
            '\n' => Token::Newline,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::NotEqual
                } else if self.peek() == Some('/') {
                    // one-line comment
                    self.advance(); // skip the second '/'
                    let mut s = String::new();
                    while let Some(c) = self.peek() {
                        if c == '\n' { break; }
                        s.push(self.advance().unwrap());
                    }
                    Token::Comment(s)
                } else if self.peek() == Some('*') {
                    // Multi-line comment
                    self.advance(); // skip '*'
                    let mut s = String::new();
                    while let Some(c) = self.advance() {
                        if c == '*' && self.peek() == Some('/') {
                            self.advance(); // skip '/'
                            break;
                        }
                        s.push(c);
                    }
                    Token::Comment(s)
                } else {
                    Token::Slash
                }
            }
            '=' => {
                if self.peek() == Some('>'){
                    self.advance();
                    Token::Arrow
                } else {
                    Token::Equal
                }

            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::LessEqual
                } else {
                    Token::Less
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }
            }
            ':' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::Assign
                } else {
                    Token::Error {
                        message: "Unexpected ':'".into(),
                        line: self.line,
                        col: self.col,
                    }
                }
            }
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            '.' => {
                if self.peek() == Some('.') {
                    self.advance();
                    Token::Range
                } else {
                    Token::Dot
                }
            }
            '"' | '\'' => self.lex_string(ch),
            c if c.is_ascii_digit() => self.lex_number(c),
            c if c.is_alphabetic() => self.lex_identifier(c),
            _ => Token::Error {
                message: format!("Unexpected character: '{}'", ch),
                line: self.line,
                col: self.col,
            },
        }
    }

    //Lexing Numbers
    fn lex_number(&mut self, first: char) -> Token {
        let mut s = first.to_string();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(self.advance().unwrap());
            } else if c == '.' {
                if self.input.get(self.pos + 1) == Some(&'.') {
                    break;
                }
                s.push(self.advance().unwrap());
                while let Some(c2) = self.peek() {
                    if c2.is_ascii_digit() {
                        s.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                return Token::Real(s.parse().unwrap());
            } else {
                break;
            }
        }
        Token::Integer(s.parse().unwrap())
    }
    //Lexing Identifiers/VarNames
    fn lex_identifier(&mut self, first: char) -> Token {
        let mut s = first.to_string();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                s.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        match s.as_str() {
            "var" => Token::Var,
            "if" => Token::If,
            "func" => Token::Func,
            "is" => Token::Is,
            "then" => Token::Then,
            "else" => Token::Else,
            "end" => Token::End,
            "while" => Token::While,
            "for" => Token::For,
            "loop" => Token::Loop,
            "exit" => Token::Exit,
            "return" => Token::Return,
            "print" => Token::Print,
            "true" => Token::True,
            "false" => Token::False,
            "none" => Token::None,
            "and" => Token::And,
            "or" => Token::Or,
            "xor" => Token::Xor,
            "not" => Token::Not,
            "in" => Token::In,
            _ => Token::Identifier(s),
        }
    }

    //Lexing Strings
    fn lex_string(&mut self, quote: char) -> Token {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            self.advance();
            if c == quote {
                break;
            }
            s.push(c);
        }
        Token::String(s)
    }
}
