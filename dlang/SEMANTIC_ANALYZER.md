# Semantic Analyzer Implementation

This document describes the semantic analyzer implementation for the language, including semantic checks and optimizations.

## Overview

The semantic analyzer consists of two main components:

1. **Semantic Checks** - Analyze the code for correctness without modifying the AST
2. **Optimizations** - Transform the AST to improve efficiency

## Implementation Files

- `src/analyzer.rs` - Contains the semantic checker and optimizer

## Semantic Checks (Non-Modifying)

### 1. Declarations Before Usage ✓

**Location:** `SemanticChecker::check_stmt()` and `SemanticChecker::check_expr()`

**Description:** Ensures that all variables and functions are declared before they are used.

**Example:**
```d
var x := y  // Error: y is used before declaration
print x
```

### 2. Correct Keyword Usage ✓

**Location:** `SemanticChecker::check_stmt()`

**Description:** Checks that `return` statements are only used inside functions (not in global scope).

**Example:**
```d
return  // Error: return statement outside of function
```

### 3. Array Bound Checking (Partial) ✓

**Location:** `SemanticChecker::check_array_bounds()`

**Description:** For array literals with constant indexes, checks if the index is within bounds.

**Example:**
```d
var arr := [1, 2, 3]
var x := arr[5]  // Error: index 5 out of bounds (array size: 3)
```

## Optimizations (Modifying AST)

### 1. Constant Expression Simplification ✓

**Location:** `Optimizer::fold_constants()` and `Optimizer::simplify_expr()`

**Description:** Simplifies constant expressions during compilation.

**Example:**
```d
// Before
var x := 5 + 3

// After
var x := 8
```

**Supported Operations:**
- Arithmetic: `+`, `-`, `*`, `/`
- Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Boolean: `&&`, `||`, `^^`
- Unary: `!`, `-`

### 2. Removing Unused Variables ✓

**Location:** `Optimizer::remove_unused_variables()`

**Description:** Removes variable declarations that are never used in the program.

**Example:**
```d
// Before
var x := 10
var y := 20
print y

// After
var y := 20
print y
```

### 3. Simplifying Conditionals ✓

**Location:** `Optimizer::simplify_conditionals()`

**Description:** Simplifies if statements with constant conditions.

**Example:**
```d
// Before
if true then
  print "hello"
else
  print "goodbye"
end

// After
print "hello"
```

### 4. Removing Unreachable Code ✓

**Location:** `Optimizer::remove_unreachable_code()`

**Description:** Removes code that appears after a return or exit statement.

**Example:**
```d
// Before
print "before"
return
print "after"  // unreachable

// After
print "before"
return
```

## Usage

The analyzer is integrated into the main program. When you run:

```bash
cargo run <filename>
```

The output will show:
1. Original AST
2. Semantic Analysis results
3. Optimized AST (if changes were made)

### Example Output

```bash
PS C:\...\dlang> cargo run test_programs/good_program.txt
--- Input ---
var x := 5 + 3
var y := x * 2
print y

--- AST ---
Original AST:
Stmts([
    VarDecl { name: "x", init: Binary { left: Integer(5), op: Add, right: Integer(3) } },
    ...
])

--- Semantic Analysis ---
✓ No semantic errors found

--- Running Optimizations ---
✓ AST was optimized

Optimized AST:
Stmts([
    VarDecl { name: "x", init: Integer(8) },  // 5 + 3 simplified to 8
    ...
])
--------------
```

## Test Programs

The directory `test_programs/` contains several test cases:

- `good_program.txt` - Tests constant folding
- `unreachable_code.txt` - Tests unreachable code removal
- `unused_var.txt` - Tests unused variable removal
- `conditional_simplify.txt` - Tests conditional simplification
- `semantic_error.txt` - Tests semantic error detection

## Requirements Met

### Minimum Requirements ✓

1. **At least 2 semantic checks that don't modify AST:**
   - ✓ Declarations Before Usage
   - ✓ Correct Keyword Usage
   - ✓ Array Bound Checking

2. **At least 2 optimizations that modify AST:**
   - ✓ Constant Expression Simplification
   - ✓ Removing Unused Variables
   - ✓ Simplifying Conditionals
   - ✓ Removing Unreachable Code

### Additional Features

- ✓ Comprehensive constant folding for all binary operations
- ✓ Unary operation folding (negation, logical not)
- ✓ Multiple optimization passes with iteration until no further changes

## Implementation Details

### Symbol Table

The `SemanticChecker` maintains a symbol table (`HashMap<String, SymbolInfo>`) to track:
- Variable declarations
- Function declarations
- Usage status

### Optimization Strategy

The `Optimizer` runs multiple optimization passes in a loop until no further changes occur:
1. Constant folding
2. Conditional simplification
3. Unreachable code removal
4. Unused variable removal

Each pass can trigger re-analysis by returning `true` if changes were made.

## Future Enhancements

Potential improvements that could be added:

1. **Type Checking** - More comprehensive type checking between function signatures and calls
2. **Function Inlining** - Replace function calls with the function body
3. **Dead Code Elimination** - Remove code that is never executed due to control flow
4. **Expression Hoisting** - Move invariant expressions out of loops
5. **Array Propagation** - Track array bounds through assignments
