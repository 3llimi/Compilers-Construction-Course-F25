use std::env;
use dlang::parser::Parser;
use dlang::analyzer::{SemanticChecker, Optimizer};
use dlang::interpreter::Interpreter;

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

            println!("+ No semantic errors found");

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

            // Run interpreter
            println!("\n--- Interpreter Execution ---");
            let mut interpreter = Interpreter::new();
            match interpreter.interpret(&ast) {
                Ok(()) => {
                    println!("+ Program executed successfully");
                }
                Err(e) => {
                    println!("-X- Runtime error: {}", e);
                }
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
        "var x := [1, 2, 3][5]",
        r#"
        if true then
          print "hello"
        else
          print "goodbye"
        end
        "#,
        r#"
        var x := 3
        var y := 10
        print (x)
        "#,
        r#"
        var x := 5 + 3
        print x
        "#,
        r#"
        print "before"
        exit
        print "after"
        "#,
        r#"
        var x := 5 + 3
        print x
        return
        "#,
        "10/0",
        r#"
        x := 3
        var x := 5
        "#,
        r#"
        var x := 3
        var x := 5
        "#,
        r#"
        var x := [1, 2, 3]
        x[5] = 10
        "#,
        r#"
        var i := 0
        while i < 3 loop //while cicle
            var j := 0
            while j < 2 loop
                var i := 3
                print i, j
                j := j + 1
            end
            i := i + 1
        end
        "#,
        r#"
        var f := func(x, y) is
            var b := x * y
            print f(3, 5)
        end
        var i := func(x, y) is
            var b := x * y
            print i(3, 2)
        end
        "#,
        r#"
        var f1 := func() is
            var x := 10
        end

        var f2 := func() is
            var x := 20
        end
        "#,
        r#"
        var x := 10
        while true loop
            var x := 20
        end
        var x := 30
        print (x)
        "#,
        r#"
        var f := func(x) => y + 1
        print f(5)
        "#,


        r#"
        var f := func(x) is
            var y := 10
            return x + y
        end

        print y

        "#,
        r#"
        var x := 5
        if true then
            var x := 10  // это разрешено (shadowing)
        end
        print x

        "#,

        r#"
        var arr := [10, 20, 30]
        print arr[1]
        print arr[0]
        print arr[3]
        print arr[4]
        "#,
        r#"
        var f := func(x, y) => x + y + z
        "#,
        "@ # $", // error case
    ];

    for s in samples { print_ast_for(s); }
}
