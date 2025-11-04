use dlang::parser::Parser;
use dlang::analyzer::{SemanticChecker, Optimizer};

use std::fs;

#[test]
fn test_good_program() {
    let source = fs::read_to_string("test_programs/good_program.txt")
        .expect("Failed to read test file");
    
    let mut parser = Parser::new(&source);
    let mut ast = parser.parse_program().unwrap();
    
    let mut checker = SemanticChecker::new();
    let result = checker.check(&ast);
    assert!(result.is_ok(), "Expected no semantic errors");
    
    let mut optimizer = Optimizer::new();
    let was_optimized = optimizer.optimize(&mut ast);
    assert!(was_optimized, "Expected optimizations to be applied");
    

}

#[test]
fn test_semantic_error() {
    let source = fs::read_to_string("test_programs/semantic_error.txt")
        .expect("Failed to read test file");
    
    let mut parser = Parser::new(&source);
    let ast = parser.parse_program().unwrap();
    
    let mut checker = SemanticChecker::new();
    let result = checker.check(&ast);
    assert!(result.is_err(), "Expected semantic error for 'y' used before declaration");
}

#[test]
fn test_conditional_simplify() {
    let source = fs::read_to_string("test_programs/conditional_simplify.txt")
        .expect("Failed to read test file");
    
    let mut parser = Parser::new(&source);
    let mut ast = parser.parse_program().unwrap();
    
    let mut optimizer = Optimizer::new();
    optimizer.optimize(&mut ast);
    
    
}

#[test]
fn test_unreachable_code_removal() {
    let source = fs::read_to_string("test_programs/unreachable_code.txt")
        .expect("Failed to read test file");
    
    let mut parser = Parser::new(&source);
    let mut ast = parser.parse_program().unwrap();
    
    let mut optimizer = Optimizer::new();
    let was_optimized = optimizer.optimize(&mut ast);
    
    assert!(was_optimized, "Expected unreachable code to be removed");
    
   
}
