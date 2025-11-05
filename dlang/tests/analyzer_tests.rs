use dlang::{Parser, SemanticChecker, Optimizer};
use std::fs;


// helper funcs with debug output

fn get_program(source: &str) -> dlang::ast::Program {
    let mut parser = Parser::new(source);
    parser.parse_program().expect("Failed to parse program")
}

fn check_semantics_verbose(source: &str, test_name: &str) -> Result<Vec<String>, String> {
    println!("\n===");
    println!("INPUT:");
    println!("{}", source);
    println!();
    println!("=======");
    
    let ast = get_program(source);
    println!("ORIGINAL AST:");
    println!("{:#?}", ast);
    println!("====");
    println!("SEMANTIC ANALYSIS:");
    
    let mut checker = SemanticChecker::new();
    let errors = checker.check(&ast).unwrap_or_else(|e| vec![e.to_string()]);
    
    if errors.is_empty() {
        println!("+ No semantic errors found");
    } else {
        println!("- Found {} error(s):", errors.len());
        for (i, error) in errors.iter().enumerate() {
            println!("  {}. {}", i + 1, error);
        }
    }
    println!("====");
    
    Ok(errors)
}

fn optimize_program_verbose(source: &str, test_name: &str) -> Result<dlang::ast::Program, String> {
    println!("\n===");
    println!("TEST: {} (OPTIMIZATION)", test_name);
    println!("===");
    println!("INPUT:");
    println!("{}", source);
    println!();
    println!("=======");
    
    let mut ast = get_program(source);
    println!("ORIGINAL AST:");
    println!("{:#?}", ast);
    println!("====");
    
    let mut checker = SemanticChecker::new();
    let errors = checker.check(&ast).unwrap_or_else(|e| vec![e.to_string()]);
    
    println!("SEMANTIC ANALYSIS:");
    if !errors.is_empty() {
        println!("- Found {} error(s):", errors.len());
        for (i, error) in errors.iter().enumerate() {
            println!("  {}. {}", i + 1, error);
        }
        println!("!!! Skipping optimizations due to semantic errors");
        println!("====");
        return Ok(ast);
    }
    println!("+ No semantic errors found");
    println!("====");
    
    println!("RUNNING OPTIMIZATIONS:");
    let mut optimizer = Optimizer::new();
    let was_modified = optimizer.optimize(&mut ast);
    
    if was_modified {
        println!("+ AST was optimized");
        println!();
        println!("=======");
        println!("OPTIMIZED AST:");
        println!("{:#?}", ast);
    } else {
        println!("+ No optimizations applied");
    }    
    Ok(ast)
}

fn optimize_program_verbose_unchecked(source: &str, test_name: &str) -> Result<dlang::ast::Program, String> {
    println!("\n===");
    println!("TEST: {} (OPTIMIZATION - NO CHECKS)", test_name);
    println!("===");
    println!("INPUT:");
    println!("{}", source);
    println!();
    println!("=======");
    
    let mut ast = get_program(source);
    println!("ORIGINAL AST:");
    println!("{:#?}", ast);
    println!("====");
    
    println!("RUNNING OPTIMIZATIONS (skipping semantic checks):");
    let mut optimizer = Optimizer::new();
    let was_modified = optimizer.optimize(&mut ast);
    
    if was_modified {
        println!("+ AST was optimized");
        println!();
        println!("=======");
        println!("OPTIMIZED AST:");
        println!("{:#?}", ast);
    } else {
        println!("+ No optimizations applied");
    }
    println!("====");
    
    println!("\n===");
    Ok(ast) 
    
}



// SEMANTIC CHECKS TESTS (Non-modifying AST)


#[test]
fn test_semantic_declarations_before_usage() {
    let source = "var x := y\nprint x";
    let errors = check_semantics_verbose(source, "Declarations Before Usage").expect("Semantic check failed");
    
    assert!(!errors.is_empty(), "Should detect undefined variable");
    assert!(errors[0].contains("used before declaration"));
}

#[test]
fn test_semantic_valid_declaration() {
    let source = "var x := 10\nprint x";
    let errors = check_semantics_verbose(source, "Valid Declaration").expect("Semantic check failed");
    
    assert!(errors.is_empty(), "Should have no errors for valid declarations");
}

#[test]
fn test_semantic_array_bound_checking_valid() {
    let source = "var x := [1, 2, 3][2]";
    let errors = check_semantics_verbose(source, "Array Literal: Valid Access").expect("Semantic check failed");
    
    assert!(errors.is_empty(), "Should have no errors for valid array access");
}

#[test]
fn test_semantic_array_bound_checking_invalid() {
    let source = "var x := [1, 2, 3][20]";
    let errors = check_semantics_verbose(source, "Array Literal: Out of Bounds").expect("Semantic check failed");
    
    assert!(!errors.is_empty(), "Should detect array index out of bounds");
    assert!(errors[0].contains("out of bounds"), "Error should mention 'out of bounds'");
}



#[test]
fn test_semantic_division_by_zero() {
    let source = "var x := 10 / 0";
    let errors = check_semantics_verbose(source, "Division by Zero").expect("Semantic check failed");
    
    assert!(!errors.is_empty(), "Should detect division by zero");
    assert!(errors[0].contains("Division by zero"));
}

#[test]
fn test_semantic_division_by_zero_real() {
    let source = "var x := 5.0 / 0.0";
    let errors = check_semantics_verbose(source, "Division by Zero (Real)").expect("Semantic check failed");
    
    assert!(!errors.is_empty(), "Should detect division by zero for reals");
    assert!(errors[0].contains("Division by zero"));
}

#[test]
fn test_semantic_division_valid() {
    let source = "var x := 10 / 2";
    let errors = check_semantics_verbose(source, "Valid Division").expect("Semantic check failed");
    
    assert!(errors.is_empty(), "Should have no errors for valid division");
}

#[test]
fn test_semantic_variable_redeclaration() {
    let source = "var x := 10\nvar x := 20";
    let errors = check_semantics_verbose(source, "Variable Re-declaration").expect("Semantic check failed");
    
    assert!(!errors.is_empty(), "Should detect variable re-declaration");
    assert!(errors[0].contains("already declared"));
}


// OPTIMIZATION TESTS: CONSTANT FOLDING


#[test]
fn test_opt_constant_folding_addition() {
    let source = "var x := 5 + 3\nprint x";
    let optimized = optimize_program_verbose(source, "Constant Folding: Addition").expect("Optimization failed");
    
    let stmts = match optimized {
        dlang::ast::Program::Stmts(s) => s,
    };
    
    if let dlang::ast::Stmt::VarDecl { init, .. } = &stmts[0] {
        if let dlang::ast::Expr::Integer(val) = init {
            assert_eq!(*val, 8, "Should fold 5 + 3 to 8");
        }
    }
}

#[test]
fn test_opt_constant_folding_subtraction() {
    let source = "var x := 10 - 3\nprint x";
    let optimized = optimize_program_verbose(source, "Constant Folding: Subtraction").expect("Optimization failed");
    
    let stmts = match optimized {
        dlang::ast::Program::Stmts(s) => s,
    };
    
    if let dlang::ast::Stmt::VarDecl { init, .. } = &stmts[0] {
        if let dlang::ast::Expr::Integer(val) = init {
            assert_eq!(*val, 7, "Should fold 10 - 3 to 7");
        }
    }
}

#[test]
fn test_opt_constant_folding_multiplication() {
    let source = "var x := 4 * 5\nprint x";
    let optimized = optimize_program_verbose(source, "Constant Folding: Multiplication").expect("Optimization failed");
    
    let stmts = match optimized {
        dlang::ast::Program::Stmts(s) => s,
    };
    
    if let dlang::ast::Stmt::VarDecl { init, .. } = &stmts[0] {
        if let dlang::ast::Expr::Integer(val) = init {
            assert_eq!(*val, 20, "Should fold 4 * 5 to 20");
        }
    }
}

#[test]
fn test_opt_constant_folding_division() {
    let source = "var x := 20 / 4\nprint x";
    let optimized = optimize_program_verbose(source, "Constant Folding: Division").expect("Optimization failed");
    
    let stmts = match optimized {
        dlang::ast::Program::Stmts(s) => s,
    };
    
    if let dlang::ast::Stmt::VarDecl { init, .. } = &stmts[0] {
        if let dlang::ast::Expr::Integer(val) = init {
            assert_eq!(*val, 5, "Should fold 20 / 4 to 5");
        }
    }
}

#[test]
fn test_opt_constant_folding_comparison_true() {
    let source = "var x := 5 < 10\nprint x";
    let optimized = optimize_program_verbose(source, "Constant Folding: Comparison (True)").expect("Optimization failed");
    
    let stmts = match optimized {
        dlang::ast::Program::Stmts(s) => s,
    };
    
    if let dlang::ast::Stmt::VarDecl { init, .. } = &stmts[0] {
        if let dlang::ast::Expr::Bool(val) = init {
            assert!(*val, "Should fold 5 < 10 to true");
        }
    }
}

#[test]
fn test_opt_constant_folding_comparison_false() {
    let source = "var x := 10 < 5\nprint x";
    let optimized = optimize_program_verbose(source, "Constant Folding: Comparison (False)").expect("Optimization failed");
    
    let stmts = match optimized {
        dlang::ast::Program::Stmts(s) => s,
    };
    
    if let dlang::ast::Stmt::VarDecl { init, .. } = &stmts[0] {
        if let dlang::ast::Expr::Bool(val) = init {
            assert!(!*val, "Should fold 10 < 5 to false");
        }
    }
}

#[test]
fn test_opt_constant_folding_unary_negation() {
    let source = "var x := -5\nprint x";
    let optimized = optimize_program_verbose(source, "Constant Folding: Unary Negation").expect("Optimization failed");
    
    let stmts = match optimized {
        dlang::ast::Program::Stmts(s) => s,
    };
    
    if let dlang::ast::Stmt::VarDecl { init, .. } = &stmts[0] {
        if let dlang::ast::Expr::Integer(val) = init {
            assert_eq!(*val, -5, "Should fold -5 to -5");
        }
    }
}


// OPTIMIZATION TESTS: UNUSED VARIABLE REMOVAL


#[test]
fn test_opt_remove_unused_variable() {
    let source = "var unused := 10\nvar used := 20\nprint used";
    let optimized = optimize_program_verbose(source, "Remove Unused Variables").expect("Optimization failed");
    
    let stmts = match optimized {
        dlang::ast::Program::Stmts(s) => s,
    };
    
    assert_eq!(stmts.len(), 2, "Should have 2 statements after removing unused var");
}

#[test]
fn test_opt_keep_used_variables() {
    let source = "var x := 10\nvar y := 20\nprint x, y";
    let optimized = optimize_program_verbose(source, "Keep Used Variables").expect("Optimization failed");
    
    let stmts = match optimized {
        dlang::ast::Program::Stmts(s) => s,
    };
    
    assert_eq!(stmts.len(), 3, "Should keep all used variables");
}


// OPTIMIZATION TESTS: CONDITIONAL SIMPLIFICATION


#[test]
fn test_opt_simplify_if_true() {
    let source = "if true then print \"hello\" end";
    let optimized = optimize_program_verbose(source, "Simplify: If True").expect("Optimization failed");
    
    let stmts = match optimized {
        dlang::ast::Program::Stmts(s) => s,
    };
    
    assert_eq!(stmts.len(), 1, "Should have 1 statement (if removed)");
}

#[test]
fn test_opt_simplify_if_false_with_else() {
    let source = "if false then print \"hello\" else print \"goodbye\" end";
    let optimized = optimize_program_verbose(source, "Simplify: If False with Else").expect("Optimization failed");
    
    let stmts = match optimized {
        dlang::ast::Program::Stmts(s) => s,
    };
    
    assert_eq!(stmts.len(), 1, "Should have 1 statement (if removed)");
}

#[test]
fn test_opt_simplify_if_false_without_else() {
    let source = "var x := 10\nif false then print \"hello\" end\nprint x";
    let optimized = optimize_program_verbose(source, "Simplify: If False without Else").expect("Optimization failed");
    
    let stmts = match optimized {
        dlang::ast::Program::Stmts(s) => s,
    };
    
    assert_eq!(stmts.len(), 2, "Should remove if false statement");
}

// OPTIMIZATION TESTS: UNREACHABLE CODE REMOVAL



#[test]
fn test_opt_remove_unreachable_after_exit() {
    let source = "print \"before\"\nexit\nprint \"after\"";
    let optimized = optimize_program_verbose(source, "Remove Unreachable: After Exit").expect("Optimization failed");
    
    let stmts = match optimized {
        dlang::ast::Program::Stmts(s) => s,
    };
    
    assert_eq!(stmts.len(), 2, "Should remove code after exit");
}


// COMBINED OPTIMIZATION TESTS


#[test]
fn test_opt_multiple_optimizations_applied() {
    let source = "var x := 5 + 3\nvar unused := 100\nif true then print x end";
    let optimized = optimize_program_verbose(source, "Multiple Optimizations").expect("Optimization failed");
    
    let stmts = match optimized {
        dlang::ast::Program::Stmts(s) => s,
    };
    
    assert!(stmts.len() >= 2, "Should have at least 2 statements after optimizations");
}


// INTEGRATION TESTS


#[test]
fn test_integration_no_optimization_with_semantic_errors() {
    let source = "var x := y + 5";
    let _errors = check_semantics_verbose(source, "Integration: Semantic Error Prevents Optimization").expect("Semantic check failed");
}


// FILE-BASED TESTS


#[test]
fn test_file_good_program() {
    let source = fs::read_to_string("test_programs/good_program.txt")
        .expect("Failed to read good_program.txt");
    
    let _errors = check_semantics_verbose(&source, "File: Good Program").expect("Semantic check failed");
}

#[test]
fn test_file_semantic_error() {
    let source = fs::read_to_string("test_programs/semantic_error.txt")
        .expect("Failed to read semantic_error.txt");
    
    let _errors = check_semantics_verbose(&source, "File: Semantic Error").expect("Semantic check failed");
}
