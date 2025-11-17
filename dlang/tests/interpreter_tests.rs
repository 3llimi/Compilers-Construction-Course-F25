use dlang::parser::Parser;
use dlang::analyzer::{SemanticChecker, Optimizer};
use dlang::interpreter::Interpreter;

/// Helper function to run interpreter tests
fn run_interpreter(source: &str) -> Result<(), String> {
    // Parse
    let mut parser = Parser::new(source);
    let mut ast = parser.parse_program()
        .map_err(|e| format!("Parse error: {}", e))?;

    // Semantic check
    let mut checker = SemanticChecker::new();
    checker.check(&ast)
        .map_err(|e| format!("Semantic error: {}", e))?;

    // Optimize
    let mut optimizer = Optimizer::new();
    optimizer.optimize(&mut ast);

    // Interpret
    let mut interpreter = Interpreter::new();
    interpreter.interpret(&ast)
        .map_err(|e| format!("Runtime error: {}", e))?;

    Ok(())
}

// ============================================
// BASIC TESTS
// ============================================

#[test]
fn test_simple_variable() {
    let source = r#"
var x := 42
print x
"#;
    assert!(run_interpreter(source).is_ok());
}

#[test]
fn test_arithmetic() {
    let source = r#"
var a := 10
var b := 20
var sum := a + b
print sum
"#;
    assert!(run_interpreter(source).is_ok());
}

#[test]
fn test_constant_folding() {
    let source = r#"
var result := 5 + 3 * 2
print result
"#;
    assert!(run_interpreter(source).is_ok());
}

#[test]
fn test_string_concat() {
    let source = r#"
var greeting := "Hello"
var name := "World"
print greeting + " " + name
"#;
    assert!(run_interpreter(source).is_ok());
}

// ============================================
// CONDITIONALS
// ============================================

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
    assert!(run_interpreter(source).is_ok());
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
    assert!(run_interpreter(source).is_ok());
}

// ============================================
// LOOPS
// ============================================

#[test]
fn test_while_loop() {
    let source = r#"
var i := 1
while i <= 5 loop
    print i
    i := i + 1
end
"#;
    assert!(run_interpreter(source).is_ok());
}

#[test]
fn test_for_loop_array() {
    let source = r#"
var numbers := [10, 20, 30]
for num in numbers loop
    print num
end
"#;
    assert!(run_interpreter(source).is_ok());
}

#[test]
fn test_for_loop_range() {
    let source = r#"
for i in 1..5 loop
    print i
end
"#;
    assert!(run_interpreter(source).is_ok());
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
    assert!(run_interpreter(source).is_ok());
}

// ============================================
// FUNCTIONS
// ============================================

#[test]
fn test_simple_function() {
    let source = r#"
var add := func(x, y) => x + y
print add(5, 3)
"#;
    assert!(run_interpreter(source).is_ok());
}



#[test]
fn test_closure() {
    let source = r#"
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
"#;
    assert!(run_interpreter(source).is_ok());
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
    assert!(run_interpreter(source).is_ok());
}

// ============================================
// ARRAYS
// ============================================

#[test]
fn test_array_access() {
    let source = r#"
var arr := [10, 20, 30]
print arr[1]
print arr[2]
print arr[3]
"#;
    assert!(run_interpreter(source).is_ok());
}

#[test]
fn test_array_assignment() {
    let source = r#"
var numbers := [10, 20, 30]
numbers[2] := 99
print numbers[2]
"#;
    assert!(run_interpreter(source).is_ok());
}

#[test]
fn test_array_out_of_bounds() {
    let source = r#"
var arr := [1, 2, 3]
print arr[10]
"#;
    assert!(run_interpreter(source).is_err());
}

// ============================================
// TUPLES
// ============================================





// ============================================
// TYPE CHECKING
// ============================================

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
    assert!(run_interpreter(source).is_ok());
}

// ============================================
// COMPLEX EXAMPLES
// ============================================



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
    assert!(run_interpreter(source).is_ok());
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
    assert!(run_interpreter(source).is_ok());
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
    assert!(run_interpreter(source).is_ok());
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
    assert!(run_interpreter(source).is_ok());
}

// ============================================
// ERROR TESTS
// ============================================

#[test]
fn test_division_by_zero() {
    let source = r#"
var x := 10 / 0
"#;
    assert!(run_interpreter(source).is_err());
}

#[test]
fn test_undefined_variable() {
    let source = r#"
print undefinedVar
"#;
    assert!(run_interpreter(source).is_err());
}

#[test]
fn test_wrong_argument_count() {
    let source = r#"
var f := func(x, y) => x + y
print f(5)
"#;
    assert!(run_interpreter(source).is_err());
}

// ============================================
// OPTIMIZATION TESTS
// ============================================

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
    // Should optimize to just print "Adult"
    assert!(run_interpreter(source).is_ok());
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
    assert!(run_interpreter(source).is_ok());
}



// ============================================
// EDGE CASES
// ============================================

#[test]
fn test_empty_array() {
    let source = r#"
var arr := []
print arr
"#;
    assert!(run_interpreter(source).is_ok());
}



#[test]
fn test_none_value() {
    let source = r#"
var x := none
print x
"#;
    assert!(run_interpreter(source).is_ok());
}

#[test]
fn test_negative_numbers() {
    let source = r#"
var x := -5
var y := -3.14
print x
print y
"#;
    assert!(run_interpreter(source).is_ok());
}
