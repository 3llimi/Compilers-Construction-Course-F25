use crate::ast::*;
use crate::lexer::Lexer;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError { pub message: String, pub line: usize, pub col: usize }

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line > 0 { write!(f, "{} (at {}:{})", self.message, self.line, self.col) } else { write!(f, "{}", self.message) }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

fn err_from_token<T>(message: String, tok: &Token) -> ParseResult<T> {
    let (line, col) = match tok { Token::Error { line, col, .. } => (*line, *col), _ => (0, 0) };
    Err(ParseError { message, line, col })
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();
        loop { let t = lexer.next_token(); let end = t == Token::EOF; tokens.push(t); if end { break; } }
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token { self.tokens.get(self.pos).unwrap_or(&Token::EOF) }
    fn advance(&mut self) -> Token { let t = self.peek().clone(); if self.pos < self.tokens.len() { self.pos += 1; } t }
    fn match_token(&mut self, expected: &Token) -> bool { if self.peek() == expected { self.advance(); true } else { false } }

    fn expect(&mut self, expected: &Token) -> ParseResult<()> {
        if self.match_token(expected) { Ok(()) } else { err_from_token(format!("Expected {:?}, got {:?}", expected, self.peek()), self.peek()) }
    }

    fn consume_trivia(&mut self) {
        loop {
            match self.peek() {
                Token::Newline => { self.advance(); }
                Token::Comment(_) => { self.advance(); }
                Token::Semicolon => { self.advance(); }
                _ => break,
            }
        }
    }

    pub fn parse_program(&mut self) -> ParseResult<Program> {
        let mut stmts = Vec::new();
        self.consume_trivia();
        while self.peek() != &Token::EOF {
            stmts.push(self.parse_stmt()?);
            self.consume_trivia();
        }
        Ok(Program::Stmts(stmts))
    }

    fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        match self.peek() {
            Token::Var => self.parse_var_decl(),
            Token::Print => self.parse_print(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::For => self.parse_for(),
            Token::Return => self.parse_return(),
            Token::Exit => { self.advance(); Ok(Stmt::Exit) }
            _ => {
                let expr = self.parse_expression()?;
                if self.match_token(&Token::Assign) {
                    let value = self.parse_expression()?;
                    Ok(Stmt::Assign { target: expr, value })
                } else {
                    Ok(Stmt::Expr(expr))
                }
            }
        }
    }

    fn parse_var_decl(&mut self) -> ParseResult<Stmt> {
        self.expect(&Token::Var)?;
        let name = match self.advance() { Token::Identifier(s) => s, t => return err_from_token(format!("Expected identifier after var, got {:?}", t), &t) };
        let init = if self.match_token(&Token::Assign) { self.parse_expression()? } else { Expr::None };
        Ok(Stmt::VarDecl { name, init })
    }

    fn parse_print(&mut self) -> ParseResult<Stmt> {
        self.expect(&Token::Print)?;
        let mut args = Vec::new();
        args.push(self.parse_expression()?);
        while self.match_token(&Token::Comma) { args.push(self.parse_expression()?); }
        Ok(Stmt::Print { args })
    }

    fn parse_if(&mut self) -> ParseResult<Stmt> {
        self.expect(&Token::If)?;
        let cond = self.parse_expression()?;
        if self.match_token(&Token::Arrow) {
            let then_branch = vec![ self.parse_stmt()? ];
            Ok(Stmt::If { cond, then_branch, else_branch: None })
        } else {
            self.expect(&Token::Then)?;
            let then_branch = self.parse_block_until(&[Token::Else, Token::End])?;
            let else_branch = if self.match_token(&Token::Else) { Some(self.parse_block_until(&[Token::End])?) } else { None };
            self.expect(&Token::End)?;
            Ok(Stmt::If { cond, then_branch, else_branch })
        }
    }

    fn parse_while(&mut self) -> ParseResult<Stmt> {
        self.expect(&Token::While)?;
        let cond = self.parse_expression()?;
        self.expect(&Token::Loop)?;
        let body = self.parse_block_until(&[Token::End])?;
        self.expect(&Token::End)?;
        Ok(Stmt::While { cond, body })
    }

    fn parse_for(&mut self) -> ParseResult<Stmt> {
        self.expect(&Token::For)?;
        
    
        let (var, iterable) = if self.peek() == &Token::Loop {
            // Infinite loop: loop ... end
            ("_".to_string(), Expr::None)
        } else {
            let var_name = if let Token::Identifier(name) = self.peek().clone() {  
                self.advance();
                name  
            } else {
                "_".to_string()
            };
            
            // check 'in'
            if self.match_token(&Token::In) {
                let iterable_expr = self.parse_expression()?;
                (var_name, iterable_expr)
            } else {
                // only expressions without 'in'
                let iterable_expr = self.parse_expression()?;
                ("_".to_string(), iterable_expr)
            }
        };
        
        self.expect(&Token::Loop)?;
        let body = self.parse_block_until(&[Token::End])?;
        self.expect(&Token::End)?;
        
        Ok(Stmt::For { var, iterable, body })
    }
    
    fn parse_block_until(&mut self, end_tokens: &[Token]) -> ParseResult<Vec<Stmt>> {
        let mut stmts = Vec::new();
        self.consume_trivia();
        while !end_tokens.contains(self.peek()) && self.peek() != &Token::EOF {
            stmts.push(self.parse_stmt()?);
            self.consume_trivia();
        }
        Ok(stmts)
    }

    fn parse_return(&mut self) -> ParseResult<Stmt> {
        self.expect(&Token::Return)?;
        match self.peek() {
            Token::End | Token::Else | Token::Loop | Token::Newline | Token::Semicolon => Ok(Stmt::Return(None)),
            _ => Ok(Stmt::Return(Some(self.parse_expression()?)))
        }
    }

    // Expression hierarchy methods per grammar
    fn parse_expression(&mut self) -> ParseResult<Expr> {
        let mut node = self.parse_relation()?;
        loop {
            match self.peek() {
                Token::Or => { self.advance(); let rhs = self.parse_relation()?; node = Expr::Binary { left: Box::new(node), op: BinOp::Or, right: Box::new(rhs) }; }
                Token::And => { self.advance(); let rhs = self.parse_relation()?; node = Expr::Binary { left: Box::new(node), op: BinOp::And, right: Box::new(rhs) }; }
                Token::Xor => { self.advance(); let rhs = self.parse_relation()?; node = Expr::Binary { left: Box::new(node), op: BinOp::Xor, right: Box::new(rhs) }; }
                _ => break,
            }
        }
        Ok(node)
    }

    fn parse_relation(&mut self) -> ParseResult<Expr> {
        let mut node = self.parse_range()?;
        match self.peek() {
            Token::Less => { self.advance(); let rhs = self.parse_factor()?; node = Expr::Binary { left: Box::new(node), op: BinOp::Lt, right: Box::new(rhs) }; }
            Token::LessEqual => { self.advance(); let rhs = self.parse_factor()?; node = Expr::Binary { left: Box::new(node), op: BinOp::Le, right: Box::new(rhs) }; }
            Token::Greater => { self.advance(); let rhs = self.parse_factor()?; node = Expr::Binary { left: Box::new(node), op: BinOp::Gt, right: Box::new(rhs) }; }
            Token::GreaterEqual => { self.advance(); let rhs = self.parse_factor()?; node = Expr::Binary { left: Box::new(node), op: BinOp::Ge, right: Box::new(rhs) }; }
            Token::Equal => { self.advance(); let rhs = self.parse_factor()?; node = Expr::Binary { left: Box::new(node), op: BinOp::Eq, right: Box::new(rhs) }; }
            Token::NotEqual => { self.advance(); let rhs = self.parse_factor()?; node = Expr::Binary { left: Box::new(node), op: BinOp::Ne, right: Box::new(rhs) }; }
            Token::Is => { self.advance(); let rhs = self.parse_factor()?; node = Expr::Binary { left: Box::new(node), op: BinOp::Is, right: Box::new(rhs) }; }
            _ => {}
        }
        Ok(node)
    }

    fn parse_range(&mut self) -> ParseResult<Expr> {
        let mut node = self.parse_factor()?;
        
        if self.match_token(&Token::Range) {
            let end = self.parse_factor()?;
            node = Expr::Range(Box::new(node), Box::new(end));
        }
        
        Ok(node)
    }
    

    fn parse_factor(&mut self) -> ParseResult<Expr> {
        let mut node = self.parse_term()?;
        loop {
            match self.peek() {
                Token::Plus => { self.advance(); let rhs = self.parse_term()?; node = Expr::Binary { left: Box::new(node), op: BinOp::Add, right: Box::new(rhs) }; }
                Token::Minus => { self.advance(); let rhs = self.parse_term()?; node = Expr::Binary { left: Box::new(node), op: BinOp::Sub, right: Box::new(rhs) }; }
                _ => break,
            }
        }
        Ok(node)
    }

    fn parse_term(&mut self) -> ParseResult<Expr> {
        let mut node = self.parse_unary()?;
        loop {
            match self.peek() {
                Token::Star => { self.advance(); let rhs = self.parse_unary()?; node = Expr::Binary { left: Box::new(node), op: BinOp::Mul, right: Box::new(rhs) }; }
                Token::Slash => { self.advance(); let rhs = self.parse_unary()?; node = Expr::Binary { left: Box::new(node), op: BinOp::Div, right: Box::new(rhs) }; }
                _ => break,
            }
        }
        Ok(node)
    }

    fn parse_unary(&mut self) -> ParseResult<Expr> {
        match self.peek() {
            Token::Plus => { self.advance(); self.parse_unary() }
            Token::Minus => { self.advance(); Ok(Expr::Unary { op: UnOp::Neg, expr: Box::new(self.parse_unary()?) }) }
            Token::Not => { self.advance(); Ok(Expr::Unary { op: UnOp::Not, expr: Box::new(self.parse_unary()?) }) }
            _ => {
                let expr = self.parse_reference_primary()?;
                
                // check operator 'is' after expression
                if self.match_token(&Token::Is) {
                    let type_ind = self.parse_type_indicator()?;
                    Ok(Expr::IsType { expr: Box::new(expr), type_ind })
                } else {
                    Ok(expr)
                }
            }
        }
    }

    fn parse_type_indicator(&mut self) -> ParseResult<TypeIndicator> {
        match self.advance() {
            Token::TypeInt => Ok(TypeIndicator::Int),        
            Token::TypeReal => Ok(TypeIndicator::Real),      
            Token::TypeBool => Ok(TypeIndicator::Bool),      
            Token::TypeString => Ok(TypeIndicator::String),  
            Token::None => Ok(TypeIndicator::None),
            Token::LBracket => {
                self.expect(&Token::RBracket)?;
                Ok(TypeIndicator::Array)
            }
            Token::LBrace => {
                self.expect(&Token::RBrace)?;
                Ok(TypeIndicator::Tuple)
            }
            Token::Func => Ok(TypeIndicator::Func),
            t => err_from_token(format!("Expected type indicator, got {:?}", t), &t),
        }
    }

    fn parse_reference_primary(&mut self) -> ParseResult<Expr> {
        let mut expr = match self.peek().clone() {
            Token::Integer(n) => { self.advance(); Expr::Integer(n) }
            Token::Real(r) => { self.advance(); Expr::Real(r) }
            Token::True => { self.advance(); Expr::Bool(true) }
            Token::False => { self.advance(); Expr::Bool(false) }
            Token::None => { self.advance(); Expr::None }
            Token::String(s) => { self.advance(); Expr::String(s) }
            Token::Identifier(s) => { self.advance(); Expr::Ident(s) }
            Token::LParen => { self.advance(); let e = self.parse_expression()?; self.expect(&Token::RParen)?; e }
            Token::LBracket => self.parse_array_literal()?,
            Token::LBrace => self.parse_tuple_literal()?,
            Token::Func => self.parse_func_literal()?,
            t => return err_from_token(format!("Unexpected token in expression: {:?}", t), &t),
        };
        loop {
            match self.peek() {
                Token::LParen => {
                    self.advance();
                    let mut args = Vec::new();
                    if self.peek() != &Token::RParen { args.push(self.parse_expression()?); while self.match_token(&Token::Comma) { args.push(self.parse_expression()?); } }
                    self.expect(&Token::RParen)?;
                    expr = Expr::Call { callee: Box::new(expr), args };
                }
                Token::LBracket => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(&Token::RBracket)?;
                    expr = Expr::Index { target: Box::new(expr), index: Box::new(index) };
                }
                Token::Dot => {
                    self.advance();
                    match self.advance() {
                        Token::Identifier(field) => { expr = Expr::Member { target: Box::new(expr), field }; }
                        Token::Integer(n) => { expr = Expr::Member { target: Box::new(expr), field: n.to_string() }; }
                        t => return err_from_token(format!("Expected identifier or integer after '.', got {:?}", t), &t),
                    }
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_array_literal(&mut self) -> ParseResult<Expr> {
        self.expect(&Token::LBracket)?;
        let mut elems = Vec::new();
        if self.peek() != &Token::RBracket { elems.push(self.parse_expression()?); while self.match_token(&Token::Comma) { elems.push(self.parse_expression()?); } }
        self.expect(&Token::RBracket)?;
        Ok(Expr::Array(elems))
    }

    fn parse_tuple_literal(&mut self) -> ParseResult<Expr> {
        self.expect(&Token::LBrace)?;
        let mut elements = Vec::new();
        
        if self.peek() != &Token::RBrace {
            loop {
                // Check if the element is named (IDENT :=)
                let name = if let Token::Identifier(id) = self.peek() {
                    let id_clone = id.clone();
                    self.advance();
                    if self.match_token(&Token::Assign) {
                        Some(id_clone)  // named el
                    } else {
                        // beginning of the expression, roll back
                        self.pos -= 1;
                        None
                    }
                } else {
                    None
                };
                
                let value = self.parse_expression()?;
                elements.push(TupleElement { name, value });
                
                if !self.match_token(&Token::Comma) { break; }
            }
        }
        
        self.expect(&Token::RBrace)?;
        Ok(Expr::Tuple(elements))
    }
    

    fn parse_func_literal(&mut self) -> ParseResult<Expr> {
        self.expect(&Token::Func)?;
        self.expect(&Token::LParen)?;
        let mut params = Vec::new();
        if self.peek() != &Token::RParen { params.push(self.expect_ident()?); while self.match_token(&Token::Comma) { params.push(self.expect_ident()?); } }
        self.expect(&Token::RParen)?;
        if self.match_token(&Token::Arrow) { let body_expr = self.parse_expression()?; Ok(Expr::Func { params, body: FuncBody::Expr(Box::new(body_expr)) }) }
        else if self.match_token(&Token::Is) { let body = self.parse_block_until(&[Token::End])?; self.expect(&Token::End)?; Ok(Expr::Func { params, body: FuncBody::Block(body) }) }
        else { err_from_token(format!("Expected '=>' or 'is' after func params, got {:?}", self.peek()), self.peek()) }
    }

    fn expect_ident(&mut self) -> ParseResult<String> { match self.advance() { Token::Identifier(s) => Ok(s), t => err_from_token(format!("Expected identifier, got {:?}", t), &t) } }
}
