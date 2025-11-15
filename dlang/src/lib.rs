pub mod token;
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod analyzer;
pub mod interpreter;


pub use parser::Parser;
pub use analyzer::{SemanticChecker, Optimizer, AnalysisError, AnalysisResult};
pub use interpreter::{Interpreter, InterpreterError, InterpreterResult};

pub use ast::{Program, Stmt, Expr, BinOp, UnOp};


#[cfg(test)]
mod parser_tests;

#[cfg(test)]
mod tests {
    use super::lexer::Lexer;
    use super::token::Token;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("var x := 42; print \"hi\"");
        use Token::*;
        assert_eq!(lexer.next_token(), Var);
        assert_eq!(lexer.next_token(), Identifier("x".into()));
        assert_eq!(lexer.next_token(), Assign);
        assert_eq!(lexer.next_token(), Integer(42));
        assert_eq!(lexer.next_token(), Semicolon);
        assert_eq!(lexer.next_token(), Print);
        assert_eq!(lexer.next_token(), String("hi".into()));
    }

    #[test]
    fn test_if_statement() {
        let mut lexer = Lexer::new(r#"if x = 10 then print "ok" end"#);
        use Token::*;
        assert_eq!(lexer.next_token(), If);
        assert_eq!(lexer.next_token(), Identifier("x".into()));
        assert_eq!(lexer.next_token(), Equal);
        assert_eq!(lexer.next_token(), Integer(10));
        assert_eq!(lexer.next_token(), Then);
        assert_eq!(lexer.next_token(), Print);
        assert_eq!(lexer.next_token(), String("ok".into()));
        assert_eq!(lexer.next_token(), End);
    }

    #[test]
    fn test_func_definition_and_call() {
        let mut lexer = Lexer::new("var f := func(x,y)=>x*y; print f(3,4)");
        use Token::*;
        assert_eq!(lexer.next_token(), Var);
        assert_eq!(lexer.next_token(), Identifier("f".into()));
        assert_eq!(lexer.next_token(), Assign);
        assert_eq!(lexer.next_token(), Func);
        assert_eq!(lexer.next_token(), LParen);
        assert_eq!(lexer.next_token(), Identifier("x".into()));
        assert_eq!(lexer.next_token(), Comma);
        assert_eq!(lexer.next_token(), Identifier("y".into()));
        assert_eq!(lexer.next_token(), RParen);
        assert_eq!(lexer.next_token(), Arrow);
        assert_eq!(lexer.next_token(), Identifier("x".into()));
        assert_eq!(lexer.next_token(), Star);
        assert_eq!(lexer.next_token(), Identifier("y".into()));
        assert_eq!(lexer.next_token(), Semicolon);
        assert_eq!(lexer.next_token(), Print);
        assert_eq!(lexer.next_token(), Identifier("f".into()));
        assert_eq!(lexer.next_token(), LParen);
        assert_eq!(lexer.next_token(), Integer(3));
        assert_eq!(lexer.next_token(), Comma);
        assert_eq!(lexer.next_token(), Integer(4));
        assert_eq!(lexer.next_token(), RParen);
    }

    #[test]
    fn test_for_loop_over_array() {
        let mut lexer = Lexer::new("for i in [1,2,3] loop print i end");
        use Token::*;
        assert_eq!(lexer.next_token(), For);
        assert_eq!(lexer.next_token(), Identifier("i".into()));
        assert_eq!(lexer.next_token(), In);
        assert_eq!(lexer.next_token(), LBracket);
        assert_eq!(lexer.next_token(), Integer(1));
        assert_eq!(lexer.next_token(), Comma);
        assert_eq!(lexer.next_token(), Integer(2));
        assert_eq!(lexer.next_token(), Comma);
        assert_eq!(lexer.next_token(), Integer(3));
        assert_eq!(lexer.next_token(), RBracket);
        assert_eq!(lexer.next_token(), Loop);
        assert_eq!(lexer.next_token(), Print);
        assert_eq!(lexer.next_token(), Identifier("i".into()));
        assert_eq!(lexer.next_token(), End);
    }

    #[test]
    fn test_comment_and_error() {
        let mut lexer = Lexer::new("// hello\n@");
        use Token::*;
        assert_eq!(lexer.next_token(), Comment(" hello".into()));
        assert_eq!(lexer.next_token(), Newline);
        match lexer.next_token() {
            Error { message, line, col } => {
                assert!(message.contains("Unexpected character"));
                assert_eq!(line, 2);
                assert_eq!(col, 2);
            }
            _ => panic!("Expected error token"),
        }
    }


}
