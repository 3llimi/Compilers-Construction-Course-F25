pub mod token;
pub mod lexer;

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

    
}
