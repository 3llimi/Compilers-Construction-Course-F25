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
        let mut var: Option<String> = None;
        if let Token::Identifier(name) = self.peek().clone() { var = Some(name); self.advance(); }
        let mut iterable = None;
        if var.is_some() && self.match_token(&Token::In) {
            iterable = Some(self.parse_expression()?);
        } else {
            iterable = Some(self.parse_expression()?);
        }
        if self.match_token(&Token::Range) {
            let start = iterable.unwrap();
            let end = self.parse_expression()?;
            iterable = Some(Expr::Range(Box::new(start), Box::new(end)));
        }
        self.expect(&Token::Loop)?;
        let body = self.parse_block_until(&[Token::End])?;
        self.expect(&Token::End)?;
        Ok(Stmt::For { var: var.unwrap_or("_".to_string()), iterable: iterable.unwrap(), body })
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
        let mut node = self.parse_factor()?;
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
            _ => self.parse_reference_primary(),
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
            Token::LBrace => self.parse_object_literal()?,
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

    fn parse_object_literal(&mut self) -> ParseResult<Expr> {
        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();
        if self.peek() != &Token::RBrace {
            loop {
                let key = match self.advance() { Token::Identifier(s) => s, t => return err_from_token(format!("Expected identifier in object key, got {:?}", t), &t) };
                self.expect(&Token::Assign)?;
                let value = self.parse_expression()?;
                fields.push((key, value));
                if self.match_token(&Token::Comma) { continue; }
                break;
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(Expr::Object(fields))
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

#[cfg(test)]
mod parser_tests {
    use super::*;

    // Auxiliary function for checking successful parsing
    fn parse_ok(input: &str) -> Program {
        let result = Parser::new(input).parse_program().expect("Parse should succeed");
        println!("\n========== INPUT ==========");
        println!("{}", input);
        println!("========== AST ==========");
        println!("{:#?}", result);
        println!("==========================\n");
        result
    }

    // Auxiliary function for checking parsing errors
    fn parse_err(input: &str) -> ParseError {
        println!("\n========== INPUT (expecting error) ==========");
        println!("{}", input);
        let error = Parser::new(input).parse_program().expect_err("Parse should fail");
        println!("========== ERROR ==========");
        println!("{}", error);
        println!("==========================\n");
        error
    }

    #[test]
    fn test_var_decl_with_init() {
        let prog = parse_ok("var x := 42");
        match &prog {
            Program::Stmts(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    Stmt::VarDecl { name, init } => {
                        assert_eq!(name, "x");
                        assert_eq!(init, &Expr::Integer(42));
                    }
                    _ => panic!("Expected VarDecl"),
                }
            }
        }
    }

    #[test]
    fn test_var_decl_without_init() {
        let prog = parse_ok("var y");
        match &prog {
            Program::Stmts(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    Stmt::VarDecl { name, init } => {
                        assert_eq!(name, "y");
                        assert_eq!(init, &Expr::None);
                    }
                    _ => panic!("Expected VarDecl"),
                }
            }
        }
    }

    #[test]
    fn test_assignment() {
        let prog = parse_ok("x := 10");
        match &prog {
            Program::Stmts(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    Stmt::Assign { target, value } => {
                        assert!(matches!(target, Expr::Ident(_)));
                        assert_eq!(value, &Expr::Integer(10));
                    }
                    _ => panic!("Expected Assign"),
                }
            }
        }
    }

    #[test]
    fn test_print_single_arg() {
        let prog = parse_ok("print \"hello\"");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::Print { args } => {
                        assert_eq!(args.len(), 1);
                        assert_eq!(args[0], Expr::String("hello".into()));
                    }
                    _ => panic!("Expected Print"),
                }
            }
        }
    }

    #[test]
    fn test_print_multiple_args() {
        let prog = parse_ok("print x, 42, \"test\"");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::Print { args } => {
                        assert_eq!(args.len(), 3);
                    }
                    _ => panic!("Expected Print"),
                }
            }
        }
    }

    #[test]
    fn test_if_then_end() {
        let prog = parse_ok("if x < 10 then print x end");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::If { cond, then_branch, else_branch } => {
                        assert!(matches!(cond, Expr::Binary { .. }));
                        assert_eq!(then_branch.len(), 1);
                        assert!(else_branch.is_none());
                    }
                    _ => panic!("Expected If"),
                }
            }
        }
    }

    #[test]
    fn test_if_then_else_end() {
        let prog = parse_ok("if x = 5 then print \"yes\" else print \"no\" end");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::If { cond, then_branch, else_branch } => {
                        assert!(matches!(cond, Expr::Binary { .. }));
                        assert_eq!(then_branch.len(), 1);
                        assert!(else_branch.is_some());
                        assert_eq!(else_branch.as_ref().unwrap().len(), 1);
                    }
                    _ => panic!("Expected If"),
                }
            }
        }
    }

    #[test]
    fn test_if_arrow_short_form() {
        let prog = parse_ok("if x > 0 => print x");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::If { cond, then_branch, else_branch } => {
                        assert!(matches!(cond, Expr::Binary { .. }));
                        assert_eq!(then_branch.len(), 1);
                        assert!(else_branch.is_none());
                    }
                    _ => panic!("Expected If"),
                }
            }
        }
    }

    #[test]
    fn test_while_loop() {
        let prog = parse_ok("while i < 10 loop i := i + 1 end");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::While { cond, body } => {
                        assert!(matches!(cond, Expr::Binary { .. }));
                        assert_eq!(body.len(), 1);
                    }
                    _ => panic!("Expected While"),
                }
            }
        }
    }

    #[test]
    fn test_for_loop_with_array() {
        let prog = parse_ok("for i in [1,2,3] loop print i end");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::For { var, iterable, body } => {
                        assert_eq!(var, "i");
                        assert!(matches!(iterable, Expr::Array(_)));
                        assert_eq!(body.len(), 1);
                    }
                    _ => panic!("Expected For"),
                }
            }
        }
    }

    #[test]
    fn test_return_with_value() {
        let prog = parse_ok("return 42");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::Return(Some(expr)) => {
                        assert_eq!(expr, &Expr::Integer(42));
                    }
                    _ => panic!("Expected Return with value"),
                }
            }
        }
    }

    #[test]
    fn test_exit() {
        let prog = parse_ok("exit");
        match &prog {
            Program::Stmts(stmts) => {
                assert!(matches!(stmts[0], Stmt::Exit));
            }
        }
    }

    #[test]
    fn test_binary_expression_precedence() {
        let prog = parse_ok("x := 2 + 3 * 4");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::Assign { value, .. } => {
                        // Должно быть: 2 + (3 * 4)
                        match value {
                            Expr::Binary { left, op, right } => {
                                assert_eq!(left.as_ref(), &Expr::Integer(2));
                                assert_eq!(op, &BinOp::Add);
                                assert!(matches!(right.as_ref(), Expr::Binary { .. }));
                            }
                            _ => panic!("Expected binary expression"),
                        }
                    }
                    _ => panic!("Expected Assign"),
                }
            }
        }
    }

    #[test]
    fn test_unary_minus() {
        let prog = parse_ok("x := -5");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::Assign { value, .. } => {
                        match value {
                            Expr::Unary { op, expr } => {
                                assert_eq!(op, &UnOp::Neg);
                                assert_eq!(expr.as_ref(), &Expr::Integer(5));
                            }
                            _ => panic!("Expected unary expression"),
                        }
                    }
                    _ => panic!("Expected Assign"),
                }
            }
        }
    }

    #[test]
    fn test_unary_not() {
        let prog = parse_ok("x := not true");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::Assign { value, .. } => {
                        match value {
                            Expr::Unary { op, expr } => {
                                assert_eq!(op, &UnOp::Not);
                                assert_eq!(expr.as_ref(), &Expr::Bool(true));
                            }
                            _ => panic!("Expected unary expression"),
                        }
                    }
                    _ => panic!("Expected Assign"),
                }
            }
        }
    }

    #[test]
    fn test_array_literal() {
        let prog = parse_ok("var arr := [1, 2, 3]");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::VarDecl { init, .. } => {
                        match init {
                            Expr::Array(elems) => {
                                assert_eq!(elems.len(), 3);
                            }
                            _ => panic!("Expected array literal"),
                        }
                    }
                    _ => panic!("Expected VarDecl"),
                }
            }
        }
    }

    #[test]
    fn test_empty_array() {
        let prog = parse_ok("var arr := []");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::VarDecl { init, .. } => {
                        match init {
                            Expr::Array(elems) => {
                                assert_eq!(elems.len(), 0);
                            }
                            _ => panic!("Expected array literal"),
                        }
                    }
                    _ => panic!("Expected VarDecl"),
                }
            }
        }
    }

    #[test]
    fn test_object_literal() {
        let prog = parse_ok("var obj := {x:=1, y:=2}");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::VarDecl { init, .. } => {
                        match init {
                            Expr::Object(fields) => {
                                assert_eq!(fields.len(), 2);
                            }
                            _ => panic!("Expected object literal"),
                        }
                    }
                    _ => panic!("Expected VarDecl"),
                }
            }
        }
    }

    #[test]
    fn test_func_arrow_syntax() {
        let prog = parse_ok("var f := func(x) => x + 1");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::VarDecl { init, .. } => {
                        match init {
                            Expr::Func { params, body } => {
                                assert_eq!(params.len(), 1);
                                assert!(matches!(body, FuncBody::Expr(_)));
                            }
                            _ => panic!("Expected func literal"),
                        }
                    }
                    _ => panic!("Expected VarDecl"),
                }
            }
        }
    }

    #[test]
    fn test_func_block_syntax() {
        let prog = parse_ok("var f := func(x, y) is return x + y end");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::VarDecl { init, .. } => {
                        match init {
                            Expr::Func { params, body } => {
                                assert_eq!(params.len(), 2);
                                assert!(matches!(body, FuncBody::Block(_)));
                            }
                            _ => panic!("Expected func literal"),
                        }
                    }
                    _ => panic!("Expected VarDecl"),
                }
            }
        }
    }

    #[test]
    fn test_function_call() {
        let prog = parse_ok("f(1, 2)");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::Expr(expr) => {
                        match expr {
                            Expr::Call { callee, args } => {
                                assert!(matches!(callee.as_ref(), Expr::Ident(_)));
                                assert_eq!(args.len(), 2);
                            }
                            _ => panic!("Expected function call"),
                        }
                    }
                    _ => panic!("Expected Expr statement"),
                }
            }
        }
    }

    #[test]
    fn test_array_indexing() {
        let prog = parse_ok("x := arr[1]");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::Assign { value, .. } => {
                        match value {
                            Expr::Index { target, index } => {
                                assert!(matches!(target.as_ref(), Expr::Ident(_)));
                                assert_eq!(index.as_ref(), &Expr::Integer(1));
                            }
                            _ => panic!("Expected index expression"),
                        }
                    }
                    _ => panic!("Expected Assign"),
                }
            }
        }
    }

    #[test]
    fn test_member_access() {
        let prog = parse_ok("x := obj.field");
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::Assign { value, .. } => {
                        match value {
                            Expr::Member { target, field } => {
                                assert!(matches!(target.as_ref(), Expr::Ident(_)));
                                assert_eq!(field, "field");
                            }
                            _ => panic!("Expected member access"),
                        }
                    }
                    _ => panic!("Expected Assign"),
                }
            }
        }
    }

    #[test]
    fn test_multiple_statements() {
        let prog = parse_ok("var x := 1\nvar y := 2\nprint x, y");
        match &prog {
            Program::Stmts(stmts) => {
                assert_eq!(stmts.len(), 3);
            }
        }
    }

    #[test]
    fn test_comments_ignored() {
        let prog = parse_ok("// comment\nvar x := 42 // another comment\n/* multi\nline */");
        match &prog {
            Program::Stmts(stmts) => {
                assert_eq!(stmts.len(), 1);
            }
        }
    }

    #[test]
    fn test_nested_blocks() {
        let prog = parse_ok(r#"
            if x > 0 then
                while y < 10 loop
                    print y
                    y := y + 1
                end
            end
        "#);
        match &prog {
            Program::Stmts(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    Stmt::If { then_branch, .. } => {
                        assert_eq!(then_branch.len(), 1);
                        assert!(matches!(then_branch[0], Stmt::While { .. }));
                    }
                    _ => panic!("Expected If"),
                }
            }
        }
    }

    #[test]
    fn test_error_missing_end() {
        let err = parse_err("if x > 0 then print x");
        assert!(err.message.contains("Expected"));
    }

    #[test]
    fn test_error_invalid_syntax() {
        let err = parse_err("var := 42");
        assert!(err.message.contains("identifier"));
    }
}
