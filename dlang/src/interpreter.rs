use crate::ast::*;
use std::collections::HashMap;

// Runtime value representation
#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Real(f64),
    Bool(bool),
    String(String),
    None,
    Array(Vec<Value>),
    Tuple(HashMap<String, Value>),  // Named fields
    Function {
        params: Vec<String>,
        body: FuncBody,
        closure: Environment,  // Captured environment for closures
    },
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Real(a), Value::Real(b)) => (a - b).abs() < f64::EPSILON,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::None, Value::None) => true,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Function { .. }, Value::Function { .. }) => false,  // Functions are never equal
            _ => false,
        }
    }
}

// Environment for variable storage with scoping
#[derive(Debug, Clone)]
pub struct Environment {
    variables: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }

    pub fn new_with_parent(parent: Environment) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.variables.get(name) {
            Some(value.clone())
        } else if let Some(ref parent) = self.parent {
            parent.get(name)
        } else {
            None
        }
    }

    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            true
        } else if let Some(ref mut parent) = self.parent {
            parent.assign(name, value)
        } else {
            false
        }
    }
}

// Interpreter errors
#[derive(Debug)]
pub enum InterpreterError {
    RuntimeError(String),
    UndefinedVariable(String),
    TypeError(String),
    DivisionByZero,
    IndexOutOfBounds { index: i64, size: usize },
    InvalidOperation(String),
    Return(Value),  // Special: return value
    Exit,           // Special: exit signal
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            InterpreterError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            InterpreterError::TypeError(msg) => write!(f, "Type error: {}", msg),
            InterpreterError::DivisionByZero => write!(f, "Division by zero"),
            InterpreterError::IndexOutOfBounds { index, size } => {
                write!(f, "Index {} out of bounds (array size: {})", index, size)
            }
            InterpreterError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            InterpreterError::Return(_) => write!(f, "Return"),
            InterpreterError::Exit => write!(f, "Exit"),
        }
    }
}

pub type InterpreterResult<T> = Result<T, InterpreterError>;

// Main interpreter
pub struct Interpreter {
    environment: Environment,
    inside_loop: bool,
    inside_function: bool,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
            inside_loop: false,
            inside_function: false,
        }
    }

    pub fn interpret(&mut self, program: &Program) -> InterpreterResult<()> {
        match program {
            Program::Stmts(stmts) => {
                for stmt in stmts {
                    self.execute_stmt(stmt)?;
                }
                Ok(())
            }
        }
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> InterpreterResult<()> {
        match stmt {
            Stmt::VarDecl { name, init } => {
                let value = self.evaluate_expr(init)?;
                self.environment.define(name.clone(), value);
                Ok(())
            }

            Stmt::Assign { target, value } => {
                let val = self.evaluate_expr(value)?;
                self.assign_to_target(target, val)?;
                Ok(())
            }

            Stmt::Print { args } => {
                let mut output = Vec::new();
                for arg in args {
                    let val = self.evaluate_expr(arg)?;
                    output.push(self.value_to_string(&val));
                }
                println!("{}", output.join(" "));
                Ok(())
            }

            Stmt::If { cond, then_branch, else_branch } => {
                let cond_val = self.evaluate_expr(cond)?;
                let cond_bool = self.value_to_bool(&cond_val)?;

                if cond_bool {
                    self.execute_block(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute_block(else_branch)?;
                }
                Ok(())
            }

            Stmt::While { cond, body } => {
                let prev_inside_loop = self.inside_loop;
                self.inside_loop = true;

                loop {
                    let cond_val = self.evaluate_expr(cond)?;
                    let cond_bool = self.value_to_bool(&cond_val)?;
                    if !cond_bool {
                        break;
                    }

                    match self.execute_block(body) {
                        Ok(()) => {}
                        Err(InterpreterError::Exit) => {
                            self.inside_loop = prev_inside_loop;
                            return Ok(());  // Exit breaks out of loop
                        }
                        Err(InterpreterError::Return(_)) => {
                            // Return propagates up
                            self.inside_loop = prev_inside_loop;
                            return Err(InterpreterError::Return(Value::None));
                        }
                        Err(e) => {
                            self.inside_loop = prev_inside_loop;
                            return Err(e);
                        }
                    }
                }

                self.inside_loop = prev_inside_loop;
                Ok(())
            }

            Stmt::For { var, iterable, body } => {
                let prev_inside_loop = self.inside_loop;
                self.inside_loop = true;

                // Handle infinite loop (when iterable is None)
                if matches!(iterable, Expr::None) {
                    loop {
                        let env_clone = self.environment.clone();
                        let old_env = std::mem::replace(&mut self.environment, Environment::new_with_parent(env_clone));
                        if var != "_" {
                            self.environment.define(var.clone(), Value::None);
                        }

                        match self.execute_block(body) {
                            Ok(()) => {}
                            Err(InterpreterError::Exit) => {
                                self.environment = old_env;
                                self.inside_loop = prev_inside_loop;
                                return Ok(());
                            }
                            Err(InterpreterError::Return(_)) => {
                                self.environment = old_env;
                                self.inside_loop = prev_inside_loop;
                                return Err(InterpreterError::Return(Value::None));
                            }
                            Err(e) => {
                                self.environment = old_env;
                                self.inside_loop = prev_inside_loop;
                                return Err(e);
                            }
                        }

                        self.environment = old_env;
                    }
                }

                // Evaluate iterable - if it's a Range, it becomes an Array
                let iterable_val = match iterable {
                    Expr::Range(low, high) => {
                        // Handle range directly
                        let low_val = self.evaluate_expr(low)?;
                        let high_val = self.evaluate_expr(high)?;
                        self.evaluate_range(&low_val, &high_val)?
                    }
                    _ => self.evaluate_expr(iterable)?,
                };
                let items = self.iterable_to_vec(&iterable_val)?;

                for item in items {
                    // Create new scope for loop variable
                    let env_clone = self.environment.clone();
                    let old_env = std::mem::replace(&mut self.environment, Environment::new_with_parent(env_clone));
                    self.environment.define(var.clone(), item);

                    match self.execute_block(body) {
                        Ok(()) => {}
                        Err(InterpreterError::Exit) => {
                            self.environment = old_env;
                            self.inside_loop = prev_inside_loop;
                            return Ok(());
                        }
                        Err(InterpreterError::Return(_)) => {
                            self.environment = old_env;
                            self.inside_loop = prev_inside_loop;
                            return Err(InterpreterError::Return(Value::None));
                        }
                        Err(e) => {
                            self.environment = old_env;
                            self.inside_loop = prev_inside_loop;
                            return Err(e);
                        }
                    }

                    // Restore environment (but keep loop var in scope for next iteration)
                    self.environment = old_env;
                }

                self.inside_loop = prev_inside_loop;
                Ok(())
            }

            Stmt::Return(expr) => {
                if !self.inside_function {
                    return Err(InterpreterError::RuntimeError("Return statement outside of function".to_string()));
                }
                let value = if let Some(expr) = expr {
                    self.evaluate_expr(expr)?
                } else {
                    Value::None
                };
                Err(InterpreterError::Return(value))
            }

            Stmt::Exit => {
                if !self.inside_loop {
                    return Err(InterpreterError::RuntimeError("Exit statement outside of loop".to_string()));
                }
                Err(InterpreterError::Exit)
            }

            Stmt::Expr(expr) => {
                self.evaluate_expr(expr)?;
                Ok(())
            }
        }
    }

    fn execute_block(&mut self, stmts: &[Stmt]) -> InterpreterResult<()> {
        let env_clone = self.environment.clone();
        let old_env = std::mem::replace(&mut self.environment, Environment::new_with_parent(env_clone));

        for stmt in stmts {
            match self.execute_stmt(stmt) {
                Ok(()) => {}
                Err(e @ InterpreterError::Return(_)) | Err(e @ InterpreterError::Exit) => {
                    // Propagate return/exit - restore env first
                    self.environment = old_env;
                    return Err(e);
                }
                Err(e) => {
                    self.environment = old_env;
                    return Err(e);
                }
            }
        }

        self.environment = old_env;
        Ok(())
    }

    fn evaluate_expr(&mut self, expr: &Expr) -> InterpreterResult<Value> {
        match expr {
            Expr::Integer(n) => Ok(Value::Integer(*n)),
            Expr::Real(n) => Ok(Value::Real(*n)),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::None => Ok(Value::None),

            Expr::Ident(name) => {
                self.environment.get(name)
                    .ok_or_else(|| InterpreterError::UndefinedVariable(name.clone()))
            }

            Expr::Binary { left, op, right } => {
                let left_val = self.evaluate_expr(left)?;
                let right_val = self.evaluate_expr(right)?;
                self.evaluate_binary_op(op, &left_val, &right_val)
            }

            Expr::Unary { op, expr } => {
                let val = self.evaluate_expr(expr)?;
                self.evaluate_unary_op(op, &val)
            }

            Expr::Call { callee, args } => {
                let callee_val = self.evaluate_expr(callee)?;
                let arg_values: Vec<Value> = args.iter()
                    .map(|arg| self.evaluate_expr(arg))
                    .collect::<Result<_, _>>()?;

                self.call_function(&callee_val, &arg_values)
            }

            Expr::Index { target, index } => {
                let target_val = self.evaluate_expr(target)?;
                let index_val = self.evaluate_expr(index)?;
                self.evaluate_index(&target_val, &index_val)
            }

            Expr::Member { target, field } => {
                let target_val = self.evaluate_expr(target)?;
                self.evaluate_member(&target_val, field)
            }

            Expr::Array(elems) => {
                let values: Vec<Value> = elems.iter()
                    .map(|elem| self.evaluate_expr(elem))
                    .collect::<Result<_, _>>()?;
                Ok(Value::Array(values))
            }

            Expr::Tuple(elems) => {
                let mut tuple = HashMap::new();
                for (i, elem) in elems.iter().enumerate() {
                    let value = self.evaluate_expr(&elem.value)?;
                    let key = if let Some(ref name) = elem.name {
                        name.clone()
                    } else {
                        // Unnamed tuple element: use index as string (1-indexed)
                        (i + 1).to_string()
                    };
                    tuple.insert(key, value);
                }
                Ok(Value::Tuple(tuple))
            }

            Expr::Range(low, high) => {
                // Range is evaluated to produce a sequence for for loops
                // For now, we'll handle it in iterable_to_vec
                let low_val = self.evaluate_expr(low)?;
                let high_val = self.evaluate_expr(high)?;
                self.evaluate_range(&low_val, &high_val)
            }

            Expr::IsType { expr, type_ind } => {
                let val = self.evaluate_expr(expr)?;
                Ok(Value::Bool(self.check_type(&val, type_ind)))
            }

            Expr::Func { params, body } => {
                // Create a closure with current environment
                Ok(Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                    closure: self.environment.clone(),
                })
            }
        }
    }

    fn evaluate_binary_op(&self, op: &BinOp, left: &Value, right: &Value) -> InterpreterResult<Value> {
        match op {
            BinOp::Add => self.add_values(left, right),
            BinOp::Sub => self.sub_values(left, right),
            BinOp::Mul => self.mul_values(left, right),
            BinOp::Div => self.div_values(left, right),
            BinOp::Eq => Ok(Value::Bool(left == right)),
            BinOp::Ne => Ok(Value::Bool(left != right)),
            BinOp::Lt => self.compare_values(left, right, |a, b| a < b),
            BinOp::Le => self.compare_values(left, right, |a, b| a <= b),
            BinOp::Gt => self.compare_values(left, right, |a, b| a > b),
            BinOp::Ge => self.compare_values(left, right, |a, b| a >= b),
            BinOp::And => {
                let left_bool = self.value_to_bool(left)?;
                if !left_bool {
                    Ok(Value::Bool(false))
                } else {
                    Ok(Value::Bool(self.value_to_bool(right)?))
                }
            }
            BinOp::Or => {
                let left_bool = self.value_to_bool(left)?;
                if left_bool {
                    Ok(Value::Bool(true))
                } else {
                    Ok(Value::Bool(self.value_to_bool(right)?))
                }
            }
            BinOp::Xor => {
                let left_bool = self.value_to_bool(left)?;
                let right_bool = self.value_to_bool(right)?;
                Ok(Value::Bool(left_bool ^ right_bool))
            }
            BinOp::Is => {
                // This should be handled by IsType expression, but handle it here too
                Err(InterpreterError::InvalidOperation("'is' operator should be used as 'expr is type'".to_string()))
            }
        }
    }

    fn evaluate_unary_op(&self, op: &UnOp, val: &Value) -> InterpreterResult<Value> {
        match op {
            UnOp::Neg => {
                match val {
                    Value::Integer(n) => Ok(Value::Integer(-n)),
                    Value::Real(n) => Ok(Value::Real(-n)),
                    _ => Err(InterpreterError::TypeError("Cannot negate non-numeric value".to_string())),
                }
            }
            UnOp::Not => {
                let bool_val = self.value_to_bool(val)?;
                Ok(Value::Bool(!bool_val))
            }
        }
    }

    fn add_values(&self, left: &Value, right: &Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real(a + b)),
            (Value::Integer(a), Value::Real(b)) => Ok(Value::Real(*a as f64 + b)),
            (Value::Real(a), Value::Integer(b)) => Ok(Value::Real(a + *b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (Value::String(a), b) => Ok(Value::String(format!("{}{}", a, self.value_to_string(b)))),
            (a, Value::String(b)) => Ok(Value::String(format!("{}{}", self.value_to_string(a), b))),
            _ => Err(InterpreterError::TypeError("Invalid operands for addition".to_string())),
        }
    }

    fn sub_values(&self, left: &Value, right: &Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real(a - b)),
            (Value::Integer(a), Value::Real(b)) => Ok(Value::Real(*a as f64 - b)),
            (Value::Real(a), Value::Integer(b)) => Ok(Value::Real(a - *b as f64)),
            _ => Err(InterpreterError::TypeError("Invalid operands for subtraction".to_string())),
        }
    }

    fn mul_values(&self, left: &Value, right: &Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real(a * b)),
            (Value::Integer(a), Value::Real(b)) => Ok(Value::Real(*a as f64 * b)),
            (Value::Real(a), Value::Integer(b)) => Ok(Value::Real(a * *b as f64)),
            _ => Err(InterpreterError::TypeError("Invalid operands for multiplication".to_string())),
        }
    }

    fn div_values(&self, left: &Value, right: &Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if *b == 0 {
                    Err(InterpreterError::DivisionByZero)
                } else {
                    Ok(Value::Integer(a / b))
                }
            }
            (Value::Real(a), Value::Real(b)) => {
                if *b == 0.0 {
                    Err(InterpreterError::DivisionByZero)
                } else {
                    Ok(Value::Real(a / b))
                }
            }
            (Value::Integer(a), Value::Real(b)) => {
                if *b == 0.0 {
                    Err(InterpreterError::DivisionByZero)
                } else {
                    Ok(Value::Real(*a as f64 / b))
                }
            }
            (Value::Real(a), Value::Integer(b)) => {
                if *b == 0 {
                    Err(InterpreterError::DivisionByZero)
                } else {
                    Ok(Value::Real(a / *b as f64))
                }
            }
            _ => Err(InterpreterError::TypeError("Invalid operands for division".to_string())),
        }
    }

    fn compare_values<F>(&self, left: &Value, right: &Value, cmp: F) -> InterpreterResult<Value>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        let left_num = self.value_to_number(left)?;
        let right_num = self.value_to_number(right)?;
        Ok(Value::Bool(cmp(left_num, right_num)))
    }

    fn value_to_number(&self, val: &Value) -> InterpreterResult<f64> {
        match val {
            Value::Integer(n) => Ok(*n as f64),
            Value::Real(n) => Ok(*n),
            _ => Err(InterpreterError::TypeError("Expected numeric value".to_string())),
        }
    }

    fn value_to_bool(&self, val: &Value) -> InterpreterResult<bool> {
        match val {
            Value::Bool(b) => Ok(*b),
            Value::Integer(n) => Ok(*n != 0),
            Value::Real(n) => Ok(*n != 0.0),
            Value::None => Ok(false),
            Value::String(s) => Ok(!s.is_empty()),
            Value::Array(arr) => Ok(!arr.is_empty()),
            Value::Tuple(tuple) => Ok(!tuple.is_empty()),
            Value::Function { .. } => Ok(true),
        }
    }

    fn value_to_string(&self, val: &Value) -> String {
        match val {
            Value::Integer(n) => n.to_string(),
            Value::Real(n) => {
                // Format to avoid unnecessary decimals
                if n.fract() == 0.0 {
                    format!("{:.0}", n)
                } else {
                    n.to_string()
                }
            }
            Value::Bool(b) => b.to_string(),
            Value::String(s) => s.clone(),
            Value::None => "none".to_string(),
            Value::Array(arr) => {
                let elems: Vec<String> = arr.iter().map(|v| self.value_to_string(v)).collect();
                format!("[{}]", elems.join(", "))
            }
            Value::Tuple(tuple) => {
                let mut pairs: Vec<String> = tuple.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.value_to_string(v)))
                    .collect();
                pairs.sort();  // For consistent output
                format!("{{{}}}", pairs.join(", "))
            }
            Value::Function { .. } => "<function>".to_string(),
        }
    }

    fn evaluate_index(&mut self, target: &Value, index: &Value) -> InterpreterResult<Value> {
        let index_num = match index {
            Value::Integer(n) => *n,
            _ => return Err(InterpreterError::TypeError("Array index must be an integer".to_string())),
        };

        match target {
            Value::Array(arr) => {
                // Arrays are 1-indexed
                if index_num < 1 || index_num > arr.len() as i64 {
                    Err(InterpreterError::IndexOutOfBounds {
                        index: index_num,
                        size: arr.len(),
                    })
                } else {
                    Ok(arr[(index_num - 1) as usize].clone())
                }
            }
            Value::Tuple(tuple) => {
                // Tuples can be indexed by number (as string) or by name
                let key = index_num.to_string();
                tuple.get(&key)
                    .cloned()
                    .ok_or_else(|| InterpreterError::RuntimeError(format!("Tuple field '{}' not found", key)))
            }
            _ => Err(InterpreterError::TypeError("Cannot index non-array/non-tuple value".to_string())),
        }
    }

    fn evaluate_member(&mut self, target: &Value, field: &str) -> InterpreterResult<Value> {
        match target {
            Value::Tuple(tuple) => {
                tuple.get(field)
                    .cloned()
                    .ok_or_else(|| InterpreterError::RuntimeError(format!("Tuple field '{}' not found", field)))
            }
            _ => Err(InterpreterError::TypeError("Cannot access member of non-tuple value".to_string())),
        }
    }

    fn evaluate_range(&self, low: &Value, high: &Value) -> InterpreterResult<Value> {
        // Range evaluation: create an array of values from low to high (inclusive)
        let low_num = match low {
            Value::Integer(n) => *n,
            _ => return Err(InterpreterError::TypeError("Range start must be an integer".to_string())),
        };
        let high_num = match high {
            Value::Integer(n) => *n,
            _ => return Err(InterpreterError::TypeError("Range end must be an integer".to_string())),
        };

        let mut values = Vec::new();
        if low_num <= high_num {
            for i in low_num..=high_num {
                values.push(Value::Integer(i));
            }
        } else {
            // Reverse range
            for i in (high_num..=low_num).rev() {
                values.push(Value::Integer(i));
            }
        }
        Ok(Value::Array(values))
    }

    fn iterable_to_vec(&mut self, val: &Value) -> InterpreterResult<Vec<Value>> {
        match val {
            Value::Array(arr) => Ok(arr.clone()),
            _ => Err(InterpreterError::TypeError("Cannot iterate over non-iterable value".to_string())),
        }
    }

    fn check_type(&self, val: &Value, type_ind: &TypeIndicator) -> bool {
        match (val, type_ind) {
            (Value::Integer(_), TypeIndicator::Int) => true,
            (Value::Real(_), TypeIndicator::Real) => true,
            (Value::Bool(_), TypeIndicator::Bool) => true,
            (Value::String(_), TypeIndicator::String) => true,
            (Value::None, TypeIndicator::None) => true,
            (Value::Array(_), TypeIndicator::Array) => true,
            (Value::Tuple(_), TypeIndicator::Tuple) => true,
            (Value::Function { .. }, TypeIndicator::Func) => true,
            _ => false,
        }
    }

    fn call_function(&mut self, callee: &Value, args: &[Value]) -> InterpreterResult<Value> {
        match callee {
            Value::Function { params, body, closure } => {
                if params.len() != args.len() {
                    return Err(InterpreterError::RuntimeError(format!(
                        "Function expects {} arguments, got {}",
                        params.len(),
                        args.len()
                    )));
                }

                // Create new environment with closure
                let old_env = std::mem::replace(&mut self.environment, closure.clone());
                let prev_inside_function = self.inside_function;
                self.inside_function = true;

                // Bind parameters
                for (param, arg) in params.iter().zip(args.iter()) {
                    self.environment.define(param.clone(), arg.clone());
                }

                // Execute function body
                let result = match body {
                    FuncBody::Expr(expr) => {
                        match self.evaluate_expr(expr) {
                            Ok(val) => Ok(val),
                            Err(InterpreterError::Return(val)) => Ok(val),
                            Err(e) => Err(e),
                        }
                    }
                    FuncBody::Block(stmts) => {
                        let mut return_val = Value::None;
                        for stmt in stmts {
                            match self.execute_stmt(stmt) {
                                Ok(()) => {}
                                Err(InterpreterError::Return(val)) => {
                                    return_val = val;
                                    break;
                                }
                                Err(e) => {
                                    self.environment = old_env;
                                    self.inside_function = prev_inside_function;
                                    return Err(e);
                                }
                            }
                        }
                        Ok(return_val)
                    }
                };

                // Restore environment and function state
                self.environment = old_env;
                self.inside_function = prev_inside_function;
                result
            }
            _ => Err(InterpreterError::TypeError("Cannot call non-function value".to_string())),
        }
    }

    fn assign_to_target(&mut self, target: &Expr, value: Value) -> InterpreterResult<()> {
        match target {
            Expr::Ident(name) => {
                if !self.environment.assign(name, value) {
                    return Err(InterpreterError::UndefinedVariable(name.clone()));
                }
                Ok(())
            }
            Expr::Index { target: arr_expr, index } => {
                let arr_val = self.evaluate_expr(arr_expr)?;
                let index_val = self.evaluate_expr(index)?;

                match arr_val {
                    Value::Array(mut arr) => {
                        let index_num = match index_val {
                            Value::Integer(n) => n,
                            _ => return Err(InterpreterError::TypeError("Array index must be an integer".to_string())),
                        };

                        if index_num < 1 || index_num > arr.len() as i64 {
                            return Err(InterpreterError::IndexOutOfBounds {
                                index: index_num,
                                size: arr.len(),
                            });
                        }

                        arr[(index_num - 1) as usize] = value;

                        // Update the array in environment
                        if let Expr::Ident(name) = arr_expr.as_ref() {
                            self.environment.define(name.clone(), Value::Array(arr));
                        } else {
                            return Err(InterpreterError::RuntimeError("Cannot assign to non-variable array".to_string()));
                        }
                        Ok(())
                    }
                    Value::Tuple(mut tuple) => {
                        let key = match index_val {
                            Value::Integer(n) => n.to_string(),
                            Value::String(s) => s,
                            _ => return Err(InterpreterError::TypeError("Tuple index must be integer or string".to_string())),
                        };

                        tuple.insert(key.clone(), value);

                        // Update the tuple in environment
                        if let Expr::Ident(name) = arr_expr.as_ref() {
                            self.environment.define(name.clone(), Value::Tuple(tuple));
                        } else {
                            return Err(InterpreterError::RuntimeError("Cannot assign to non-variable tuple".to_string()));
                        }
                        Ok(())
                    }
                    _ => Err(InterpreterError::TypeError("Cannot assign to non-array/non-tuple value".to_string())),
                }
            }
            Expr::Member { target, field } => {
                let tuple_val = self.evaluate_expr(target)?;

                match tuple_val {
                    Value::Tuple(mut tuple) => {
                        tuple.insert(field.clone(), value);

                        // Update the tuple in environment
                        if let Expr::Ident(name) = target.as_ref() {
                            self.environment.define(name.clone(), Value::Tuple(tuple));
                        } else {
                            return Err(InterpreterError::RuntimeError("Cannot assign to non-variable tuple".to_string()));
                        }
                        Ok(())
                    }
                    _ => Err(InterpreterError::TypeError("Cannot assign to member of non-tuple value".to_string())),
                }
            }
            _ => Err(InterpreterError::RuntimeError("Invalid assignment target".to_string())),
        }
    }
}
