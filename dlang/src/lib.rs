pub mod token;
pub mod lexer;

#[cfg(test)]
mod tests {
    use super::lexer::Lexer;
    use super::token::Token;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("var x := 42; print \"hi\"");
        assert_eq!(lexer.next_token(), Token::Var);
        assert_eq!(lexer.next_token(), Token::Identifier("x".into()));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Integer(42));
        assert_eq!(lexer.next_token(), Token::Semicolon);
        assert_eq!(lexer.next_token(), Token::Print);
        assert_eq!(lexer.next_token(), Token::String("hi".into()));
    }
}
