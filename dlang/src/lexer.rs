use crate::token::Token;
//Lexer Struct
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}
//Lexer Constructor
impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }
    //Peeking and Advancing through the code
    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).cloned()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }
    //Skipping WhiteSpaces
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
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
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::NotEqual
                } else {
                    Token::Slash
                }
            }
            '=' => Token::Equal,
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
                    // language doesn’t use bare `:`
                    Token::EOF
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
            _ => Token::EOF, // placeholder for now
        }
    }
    //Lexing Numbers
    fn lex_number(&mut self, first: char) -> Token {
        let mut s = first.to_string();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(self.advance().unwrap());
            } else if c == '.' {
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
