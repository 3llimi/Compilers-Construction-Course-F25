use dlang::parser::Parser;
use dlang::analyzer::{SemanticChecker, Optimizer};
use dlang::interpreter::Interpreter;

/// Helper function to run interpreter tests with formatted output
fn run_test_formatted(test_name: &str, source: &str) -> Result<(), String> {
    println!("\n----------------------------");
    println!("TEST: {}", test_name);
    println!("----------------------------");
    println!("INPUT:");
    for line in source.trim().lines() {
        println!("  {}", line);
    }
    println!("\nOUTPUT:");
    
    // Parse
    let mut parser = Parser::new(source);
    let mut ast = parser.parse_program()
        .map_err(|e| {
            let err = format!("Parse error: {}", e);
            println!("\n  {}", err);
            println!("----------------------------\n");
            err
        })?;

    // Semantic check
    let mut checker = SemanticChecker::new();
    checker.check(&ast)
        .map_err(|e| {
            let err = format!("Semantic error: {}", e);
            println!("\n  {}", err);
            println!("----------------------------\n");
            err
        })?;

    // Optimize
    let mut optimizer = Optimizer::new();
    optimizer.optimize(&mut ast);

    // Interpret
    let mut interpreter = Interpreter::new();
    interpreter.interpret(&ast)
        .map_err(|e| {
            let err = format!("Runtime error: {}", e);
            println!("\n  {}", err);
            println!("----------------------------\n");
            err
        })?;
    
    println!("\n  PASSED");
    println!("----------------------------\n");

    Ok(())
}


/// Helper for tests that should fail
fn run_test_formatted_error(test_name: &str, source: &str) -> bool {
    println!("\n----------------------------");
    println!("TEST: {}", test_name);
    println!("----------------------------");
    println!("INPUT:");
    for line in source.trim().lines() {
        println!("  {}", line);
    }
    println!("\nEXPECTED: ERROR");
    
    // Parse
    let mut parser = Parser::new(source);
    let mut ast = match parser.parse_program() {
        Ok(ast) => ast,
        Err(e) => {
            println!("\nERROR: {}", e);
            println!("\n PASSED (Error detected as expected)");
            println!("----------------------------\n");
            return true;
        }
    };

    // Semantic check
    let mut checker = SemanticChecker::new();
    if let Err(e) = checker.check(&ast) {
        println!("\nERROR: {}", e);
        println!("\n  PASSED (Error detected as expected)");
        println!("----------------------------\n");
        return true;
    }

    // Optimize
    let mut optimizer = Optimizer::new();
    optimizer.optimize(&mut ast);

    // Interpret
    let mut interpreter = Interpreter::new();
    if let Err(e) = interpreter.interpret(&ast) {
        println!("\nERROR: {}", e);
        println!("\n  PASSED (Error detected as expected)");
        println!("----------------------------\n");
        return true;
    }
    
    println!("\n  FAILED (Expected error, but succeeded)");
    println!("----------------------------\n");
    false
}

// ========
// BASIC TESTS
// ========

#[test]
fn test_simple_variable() {
    let source = r#"
var x := 42
print x
"#;
    assert!(run_test_formatted("Simple Variable", source).is_ok());
}

#[test]
fn test_arithmetic() {
    let source = r#"
var a := 10
var b := 20
var sum := a + b
print sum
"#;
    assert!(run_test_formatted("Arithmetic", source).is_ok());
}

#[test]
fn test_constant_folding() {
    let source = r#"
var result := 5 + 3 * 2
print result
"#;
    assert!(run_test_formatted("Constant Folding", source).is_ok());
}

#[test]
fn test_string_concat() {
    let source = r#"
var greeting := "Hello"
var name := "World"
print greeting + " " + name
"#;
    assert!(run_test_formatted("String Concatenation", source).is_ok());
}

// ========
// CONDITIONALS
// ========

#[test]
fn test_if_else() {
    let source = r#"
var age := 18
if age >= 18 then
    print "Adult"
else
    print "Minor"
end
"#;
    assert!(run_test_formatted("If-Else", source).is_ok());
}

#[test]
fn test_nested_if() {
    let source = r#"
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
"#;
    assert!(run_test_formatted("Nested If", source).is_ok());
}

// ========
// LOOPS
// ========

#[test]
fn test_while_loop() {
    let source = r#"
var i := 1
while i <= 5 loop
    print i
    i := i + 1
end
"#;
    assert!(run_test_formatted("While Loop", source).is_ok());
}

#[test]
fn test_for_loop_array() {
    let source = r#"
var numbers := [10, 20, 30]
for num in numbers loop
    print num
end
"#;
    assert!(run_test_formatted("For Loop (Array)", source).is_ok());
}

#[test]
fn test_for_loop_range() {
    let source = r#"
for i in 1..5 loop
    print i
end
"#;
    assert!(run_test_formatted("For Loop (Range)", source).is_ok());
}

#[test]
fn test_exit_loop() {
    let source = r#"
var count := 0
while true loop
    print count
    count := count + 1
    if count >= 3 then
        exit
    end
end
"#;
    assert!(run_test_formatted("Exit Loop", source).is_ok());
}

// ========
// FUNCTIONS
// ========

#[test]
fn test_simple_function() {
    let source = r#"
var add := func(x, y) => x + y
print add(5, 3)
"#;
    assert!(run_test_formatted("Simple Function", source).is_ok());
}


#[test]
fn test_nested_function() {
    let source = r#"
var outer := func(x) is
    var inner := func(y) => y * 2
    return inner(x) + 10
end
print outer(5)
"#;
    assert!(run_test_formatted("Nested Function", source).is_ok());
}

// ========
// ARRAYS
// ========

#[test]
fn test_array_access() {
    let source = r#"
var arr := [10, 20, 30]
print arr[1]
print arr[2]
print arr[3]
"#;
    assert!(run_test_formatted("Array Access", source).is_ok());
}

#[test]
fn test_array_assignment() {
    let source = r#"
var numbers := [10, 20, 30]
numbers[2] := 99
print numbers[2]
"#;
    assert!(run_test_formatted("Array Assignment", source).is_ok());
}

#[test]
fn test_array_out_of_bounds() {
    let source = r#"
var arr := [1, 2, 3]
print arr[10]
"#;
    assert!(run_test_formatted_error("Array Out of Bounds", source));
}

// ========
// TUPLES
// ========

#[test]
fn test_tuple_access() {
    let source = r#"
var point := {x := 10, y := 20}
print point.x
print point.y
"#;
    assert!(run_test_formatted("Tuple Access", source).is_ok());
}

#[test]
fn test_tuple_indexed_access() {
    let source = r#"
var tuple := {a := 1, b := 2, c := 3}
print tuple.1
print tuple.2
print tuple.3
"#;
    assert!(run_test_formatted("Tuple Indexed Access", source).is_ok());
}

#[test]
fn test_tuple_concatenation() {
    let source = r#"
var t1 := {a := 1, b := 2}
var t2 := {c := 3}
var t3 := t1 + t2
print t3.a
print t3.c
"#;
    assert!(run_test_formatted("Tuple Concatenation", source).is_ok());
}

#[test]
fn test_tuple_assignment() {
    let source = r#"
var person := {name := "Alice", age := 30}
print person.age
person.age := 31
print person.age
"#;
    assert!(run_test_formatted("Tuple Assignment", source).is_ok());
}

#[test]
fn test_tuple_mixed_elements() {
    let source = r#"
var t := {a := 1, 2, c := 3}
print t.a
print t.2
print t.c
"#;
    assert!(run_test_formatted("Tuple Mixed Elements", source).is_ok());
}

#[test]
fn test_empty_tuple() {
    let source = r#"
var t := {}
print t
"#;
    assert!(run_test_formatted("Empty Tuple", source).is_ok());
}

// ========
// TYPE CHECKING
// ========

#[test]
fn test_type_checking() {
    let source = r#"
var x := 42
if x is int then
    print "integer"
end

var y := 3.14
if y is real then
    print "real"
end

var z := "hello"
if z is string then
    print "string"
end
"#;
    assert!(run_test_formatted("Type Checking", source).is_ok());
}

// ========
// COMPLEX EXAMPLES
// ========

#[test]
fn test_find_max() {
    let source = r#"
var numbers := [23, 67, 12, 89, 45]
var max := numbers[1]

for num in numbers loop
    if num > max then
        max := num
    end
end

print max
"#;
    assert!(run_test_formatted("Find Maximum", source).is_ok());
}

#[test]
fn test_iterative_factorial() {
    let source = r#"
var n := 5
var result := 1
var i := 1

while i <= n loop
    result := result * i
    i := i + 1
end

print result
"#;
    assert!(run_test_formatted("Iterative Factorial", source).is_ok());
}

#[test]
fn test_calculator() {
    let source = r#"
var add := func(a, b) => a + b
var sub := func(a, b) => a - b
var mul := func(a, b) => a * b
var div := func(a, b) => a / b

var x := 10
var y := 3

print add(x, y)
print sub(x, y)
print mul(x, y)
print div(x, y)
"#;
    assert!(run_test_formatted("Calculator", source).is_ok());
}

#[test]
fn test_shadowing() {
    let source = r#"
var x := 100

if true then
    var x := 200
    print x
end

print x
"#;
    assert!(run_test_formatted("Variable Shadowing", source).is_ok());
}

// ========
// ERROR TESTS
// ========

#[test]
fn test_division_by_zero() {
    let source = r#"
var x := 10 / 0
"#;
    assert!(run_test_formatted_error("Division by Zero", source));
}

#[test]
fn test_undefined_variable() {
    let source = r#"
print undefinedVar
"#;
    assert!(run_test_formatted_error("Undefined Variable", source));
}

#[test]
fn test_wrong_argument_count() {
    let source = r#"
var f := func(x, y) => x + y
print f(5)
"#;
    assert!(run_test_formatted_error("Wrong Argument Count", source));
}

// ========
// OPTIMIZATION TESTS
// ========

#[test]
fn test_constant_propagation() {
    let source = r#"
var age := 18
if age >= 18 then
    print "Adult"
else
    print "Minor"
end
"#;
    assert!(run_test_formatted("Constant Propagation", source).is_ok());
}

#[test]
fn test_dead_code_elimination() {
    let source = r#"
if false then
    print "This won't print"
end

if true then
    print "This will print"
end
"#;
    assert!(run_test_formatted("Dead Code Elimination", source).is_ok());
}

// ========
// EDGE CASES
// ========

#[test]
fn test_empty_array() {
    let source = r#"
var arr := []
print arr
"#;
    assert!(run_test_formatted("Empty Array", source).is_ok());
}

#[test]
fn test_none_value() {
    let source = r#"
var x := none
print x
"#;
    assert!(run_test_formatted("None Value", source).is_ok());
}

#[test]
fn test_negative_numbers() {
    let source = r#"
var x := -5
var y := -3.14
print x
print y
"#;
    assert!(run_test_formatted("Negative Numbers", source).is_ok());
}

#[test]
fn test_boolean_simplification() {
    let source = r#"
var x := true
var y := true
var z := false

print x
print y
print z
"#;
    assert!(run_test_formatted("Boolean Simplification", source).is_ok());
}

#[test]
fn test_fibonacci() {
    let source = r#"
var fib := func(n) is
    if n <= 1 then
        return n
    end
    return fib(n - 1) + fib(n - 2)
end

print fib(1)
print fib(5)
"#;
    assert!(run_test_formatted("Fibonacci", source).is_ok());
}

#[test]
fn test_recursive_factorial() {
    let source = r#"
var factorial := func(n) is
    if n <= 1 then
        return 1
    end
    return n * factorial(n - 1)
end

print factorial(5)
"#;
    assert!(run_test_formatted("Recursive Factorial", source).is_ok());
}
