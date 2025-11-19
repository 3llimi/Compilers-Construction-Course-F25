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
            // ============================================
    // topic 1: base operations
    // ============================================
    
    // 1.1 vars and arifmethic
    r#"
    // Простые переменные
    var x := 10
    var y := 20
    print x + y
    "#,
    
        // 1.2 Constant folding в действии
        r#"
    // Оптимизация константных выражений
    var result := 5 + 3 * 2
    print result
    "#,
    
        // 1.3 work с real numbers
        r#"
    var pi := 3.14
    var radius := 5.0
    var area := pi * radius * radius
    print area
    "#,
    
        // 1.4 strings
        r#"
    var greeting := "Hello"
    var name := "World"
    print greeting + " " + name + "!"
    "#,
    
        // ============================================
        // topic 2: condititons
        // ============================================
        
        // 2.1 simple conditions
        r#"
    var age := 18
    if age >= 18 then
        print "Adult"
    else
        print "Minor"
    end
    "#,
    
        // 2.2 nested conditions
        r#"
    var score := 85
    if score >= 90 then
        print "A"
    else
        if score >= 80 then
            print "B"
        else
            print "C"
        end
    end
    "#,
    
        // 2.3 optimization if (true)
        r#"
    if true then
        print "This will print"
    end
    
    if false then
        print "This won't print"
    end
    "#,
    
        // ============================================
        // topic 3: cycles
        // ============================================
        
        // 3.1 While cycle
        r#"
    var i := 1
    while i <= 5 loop
        print i
        i := i + 1
    end
    "#,
    
        // 3.2 For cycle with an array
        r#"
    var numbers := [10, 20, 30, 40, 50]
    for num in numbers loop
        print num
    end
    "#,
    
        // 3.3 For cycle with range
        r#"
    for i in 1..5 loop
        print i
    end
    "#,
    
        // 3.4 nested cycle
        r#"
    for i in 1..3 loop
        for j in 1..3 loop
            print i * j
        end
    end
    "#,
    
        
    
        // ============================================
        // topic 4: funcs
        // ============================================
        
        // 4.1 simple func
        r#"
    var add := func(x, y) => x + y
    print add(5, 3)
    "#,
    
        // 4.2 func with block
        r#"
    var factorial := func(n) is
        if n <= 1 then
            return 1
        else
            return n * factorial(n - 1)
        end
    end
    
    print factorial(5)
    "#,
    
        // 4.3 func with few operators
        r#"
    var greet := func(name) is
        print "Hello, " + name + "!"
        return name
    end
    
    var result := greet("Alice")
    print "Returned: " + result
    "#,
    
        // 4.4 Closure (замыкание)
        r#"
    var makeCounter := func() is
        var count := 0
        return func() is
            count := count + 1
            return count
        end
    end
    
    var counter := makeCounter()
    print counter()
    print counter()
    print counter()
    "#,
    
        // 4.5 nested funcs
        r#"
    var outer := func(x) is
        var inner := func(y) => y * 2
        return inner(x) + 10
    end
    
    print outer(5)
    "#,
    
        // ============================================
        // topic 5: array
        // ============================================
        
        // 5.1 creation and accessing
        r#"
    var arr := [1, 2, 3, 4, 5]
    print arr[1]
    print arr[3]
    print arr[5]
    "#,
    
        // 5.2 changing the elements
        r#"
    var numbers := [10, 20, 30]
    print numbers[2]
    numbers[2] := 99
    print numbers[2]
    "#,
    
        // 5.3 array inside the cycle
        r#"
    var squares := [1, 4, 9, 16, 25]
    for sq in squares loop
        print sq
    end
    "#,
    
        // 5.4 calculating the sum of an arr elements
        r#"
    var data := [5, 10, 15, 20]
    var sum := 0
    for val in data loop
        sum := sum + val
    end
    print "Sum: " + sum
    "#,
    
        // ============================================
        // topic 6: tuples
        // ============================================
        
        // Tuple example
    r#"
    var point := {x := 10, y := 20}
    print point.x
    print point.y
    "#,

    // Tuple with indexes
    r#"
    var t := {a := 1, 2, c := 3}
    print t.a
    print t.2
    print t.c
    "#,

    
        // ============================================
        // topic 7: types and IS
        // ============================================
        
        // 7.1 check types
        r#"
    var x := 42
    var y := 3.14
    var z := "hello"
    
    if x is int then
        print "x is integer"
    end
    
    if y is real then
        print "y is real"
    end
    
    if z is string then
        print "z is string"
    end
    "#,
    
        // 7.2 check types of funcs
        r#"
    var f := func(x) => x + 1
    if f is func then
        print "f is a function"
    end
    "#,
    
        // ============================================
        // topic 8: complex examples
        // ============================================
        
        // 8.1 Fibonacci
        r#"
    var fib := func(n) is
        if n <= 1 then
            return n
        else
            return fib(n - 1) + fib(n - 2)
        end
    end
    
    print "Fibonacci numbers:"
    for i in 1..10 loop
        print fib(i)
    end
    "#,
    
        // 8.2 earch max in an array
        r#"
    var numbers := [23, 67, 12, 89, 45]
    var max := numbers[1]
    
    for num in numbers loop
        if num > max then
            max := num
        end
    end
    
    print "Maximum: " + max
    "#,
    
        // 8.4 calculating factorial iteratively
        r#"
    var n := 5
    var result := 1
    var i := 1
    
    while i <= n loop
        result := result * i
        i := i + 1
    end
    
    print "Factorial of " + n + " is " + result
    "#,
    
        // 8.5 calculators with funcs
        r#"
    var add := func(a, b) => a + b
    var sub := func(a, b) => a - b
    var mul := func(a, b) => a * b
    var div := func(a, b) => a / b
    
    var x := 10
    var y := 3
    
    print "Addition: " + add(x, y)
    print "Subtraction: " + sub(x, y)
    print "Multiplication: " + mul(x, y)
    print "Division: " + div(x, y)
    "#,
    
        // 8.6 nested scope (shadowing)
        r#"
    var x := 100
    
    if true then
        var x := 200
        print "Inner x: " + x
    end
    
    print "Outer x: " + x
    "#,
    
    
        // ============================================
        // topic 9: error handling
        // ============================================
        
        // 9.1 Division by zero (runtime error)
        r#"
    var x := 10
    var y := 0
    print x / y
    "#,
    
        // 9.2 Array out of bounds
        r#"
    var arr := [1, 2, 3]
    print arr[10]
    "#,
    
        // 9.3 Undefined variable (semantic error)
        r#"
    print undefinedVar
    "#,
    
        // 9.4 Wrong number of arguments
        r#"
    var f := func(x, y) => x + y
    print f(5)
    "#,
    
        // ============================================
        // topic 10: optimizations demonstrations
        // ============================================
        
        // 10.1 Constant folding
        r#"
    var a := 2 + 3 * 4
    var b := (10 - 5) * 2
    var c := 100 / 10 + 5
    print a
    print b
    print c
    "#,
    
        // 10.2 Dead code elimination
        r#"
    if false then
        print "This will be removed by optimizer"
    end
    
    if true then
        print "This stays"
    end
    "#,
    
        // 10.3 Unused variable removal
        r#"
    var unused := 123
    var used := 456
    print used
    "#,
    
        // 10.4 Boolean simplification
    r#"
    var n := 5
    var result := 1
    var i := 1

    while i <= n loop
        result := result * i
        i := i + 1
    end

    print result
    "#,
    ];

    for s in samples { print_ast_for(s); }
}
