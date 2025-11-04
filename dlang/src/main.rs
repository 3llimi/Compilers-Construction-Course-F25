use std::env;
use dlang::parser::Parser;
use dlang::analyzer::{SemanticChecker, Optimizer};

fn print_ast_for(input: &str) {
    println!("--- Input ---\n{}\n--- AST ---", input);
    let mut parser = Parser::new(input);
    match parser.parse_program() {
        Ok(mut ast) => {
            println!("Original AST:\n{:#?}", ast);

            // Run semantic checks
            println!("\n--- Semantic Analysis ---");
            let mut checker = SemanticChecker::new();
            
   
            let errors = match checker.check(&ast) {
                Ok(errs) => errs,
                Err(e) => {
                    println!("-X- Semantic analysis failed: {}", e);
                    println!("\n!!!  Skipping optimizations due to semantic errors");
                    println!("--------------\n");
                    return;  
                }
            };

            if !errors.is_empty() {
                println!("-X- Found {} semantic error(s):", errors.len());
                for (i, error) in errors.iter().enumerate() {
                    println!("  {}. {}", i + 1, error);
                }
                println!("\n!!!  Skipping optimizations due to semantic errors");
                println!("--------------\n");
                return;  
            }

            println!("✓ No semantic errors found");

            // Run optimizations
            println!("\n--- Running Optimizations ---");
            let mut optimizer = Optimizer::new();
            let modified = optimizer.optimize(&mut ast);

            if modified {
                println!("+ AST was optimized");
                println!("\nOptimized AST:\n{:#?}", ast);
            } else {
                println!("+ No optimizations applied");
            }
        },
        Err(e) => println!("Parse error: {}", e),
    }
    println!("--------------\n");
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        // read file (first arg)
        let path = &args[1];
        match std::fs::read_to_string(path) {
            Ok(src) => print_ast_for(&src),
            Err(e) => eprintln!("Failed to read {}: {}", path, e),
        }
        return;
    }

    // default demo snippets
    let samples = vec![
        "var x := 42",
        r#"if x < 10 then print "small" else print "big" end"#,
        "var f := func(x)=>x+1",
        "arr[1] := {x:=2,y:=3}.y",
        "for i in [1,2,3] loop print i end",
        // range not yet implemented in parser as an operator; keep arrays demo
        r#"
        var i := 0
        while i < 3 loop //while cicle
            var j := 0
            while j < 2 loop
                print i, j
                j := j + 1
            end
            i := i + 1
        end
        "#,
        r#"
        var outer := func(x) is
            var inner := func(y) => y + 1
            return inner(x)
        end
        print outer(10)
        "#,
        r#"
        var f := func(x, y) => x * y
        print f(3, 4)
        "#,
        "((t.1).2).3",
        "1.2.3",
        "1.2",
        "10/0",
        "@ # $", // error case
    ];

    for s in samples { print_ast_for(s); }
}
