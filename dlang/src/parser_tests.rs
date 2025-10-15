use crate::ast::*;
use crate::parser::*;



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
fn test_tuple_literal() {  
    let prog = parse_ok("var obj := {x:=1, y:=2}");
    match &prog {
        Program::Stmts(stmts) => {
            match &stmts[0] {
                Stmt::VarDecl { init, .. } => {
                    match init {
                        Expr::Tuple(elements) => {  
                            assert_eq!(elements.len(), 2);
                        }
                        _ => panic!("Expected Tuple literal"),
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

#[test]
fn test_range_in_for_loop_basic() {
    let input = r#"
for i in 1..10 loop
    print i
end
"#;
    let prog = parse_ok(input);
    
    match &prog {
        Program::Stmts(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0] {
                Stmt::For { var, iterable, body } => {
                    assert_eq!(var, "i");
                    
                
                    match iterable {
                        Expr::Range(start, end) => {
                            assert_eq!(start.as_ref(), &Expr::Integer(1));
                            assert_eq!(end.as_ref(), &Expr::Integer(10));
                        }
                        _ => panic!("Expected Range expression in for loop, got {:?}", iterable),
                    }
                    
                 
                    assert_eq!(body.len(), 1);
                    assert!(matches!(body[0], Stmt::Print { .. }));
                }
                _ => panic!("Expected For statement"),
            }
        }
    }
}

#[test]
fn test_range_as_expression_in_variable() {
    let input = "var range := 1..100";
    let prog = parse_ok(input);
    
    match &prog {
        Program::Stmts(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0] {
                Stmt::VarDecl { name, init } => {
                    assert_eq!(name, "range");
                   
                    match init {
                        Expr::Range(start, end) => {
                            assert_eq!(start.as_ref(), &Expr::Integer(1));
                            assert_eq!(end.as_ref(), &Expr::Integer(100));
                        }
                        _ => panic!("Expected Range expression in var decl, got {:?}", init),
                    }
                }
                _ => panic!("Expected VarDecl"),
            }
        }
    }
}


#[test]
fn test_range_in_function_call() {
   
    let input = "process(1..10)";
    let prog = parse_ok(input);
    
    match &prog {
        Program::Stmts(stmts) => {
            match &stmts[0] {
                Stmt::Expr(expr) => {
                    match expr {
                        Expr::Call { callee, args } => {
                            assert_eq!(args.len(), 1);
                            
                         
                            match &args[0] {
                                Expr::Range(start, end) => {
                                    assert_eq!(start.as_ref(), &Expr::Integer(1));
                                    assert_eq!(end.as_ref(), &Expr::Integer(10));
                                }
                                _ => panic!("Expected Range as function argument"),
                            }
                        }
                        _ => panic!("Expected Call"),
                    }
                }
                _ => panic!("Expected Expr statement"),
            }
        }
    }
}



#[test]
fn test_is_operator_with_int_type() {
    let input = "if x is int then print x end";
    let prog = parse_ok(input);
    
    match &prog {
        Program::Stmts(stmts) => {
            match &stmts[0] {
                Stmt::If { cond, .. } => {
                    match cond {
                        Expr::IsType { expr, type_ind } => {
                            
                            match expr.as_ref() {
                                Expr::Ident(name) => assert_eq!(name, "x"),
                                _ => panic!("Expected Ident"),
                            }
                            
                    
                            assert_eq!(type_ind, &TypeIndicator::Int);
                        }
                        _ => panic!("Expected IsType expression, got {:?}", cond),
                    }
                }
                _ => panic!("Expected If statement"),
            }
        }
    }
}

#[test]
fn test_is_operator_with_all_basic_types() {
    let tests = vec![
        ("x is int", TypeIndicator::Int),
        ("y is real", TypeIndicator::Real),
        ("z is bool", TypeIndicator::Bool),
        ("s is string", TypeIndicator::String),
        ("n is none", TypeIndicator::None),
    ];
    
    for (input, expected_type) in tests {
        let prog = Parser::new(input).parse_program().unwrap();
        
        match &prog {
            Program::Stmts(stmts) => {
                match &stmts[0] {
                    Stmt::Expr(expr) => {
                        match expr {
                            Expr::IsType { type_ind, .. } => {
                                assert_eq!(type_ind, &expected_type, "Failed for input: {}", input);
                            }
                            _ => panic!("Expected IsType for: {}", input),
                        }
                    }
                    _ => panic!("Expected Expr statement for: {}", input),
                }
            }
        }
    }
}
