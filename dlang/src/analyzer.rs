use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub declared: bool,
    pub used: bool,
    pub is_function: bool,
}

#[derive(Debug)]
pub enum AnalysisError {
    Message(String),
}

impl std::fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalysisError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

pub type AnalysisResult<T> = Result<T, AnalysisError>;

// ============================================================================
// PART 1: SEMANTIC CHECKS (Don't modify AST)
// ============================================================================

pub struct SemanticChecker {
    variables: HashMap<String, SymbolInfo>,
    functions: HashMap<String, SymbolInfo>,
    inside_function: bool,
    inside_loop: bool,
    errors: Vec<String>,
}

impl SemanticChecker {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            inside_function: false,
            inside_loop: false,
            errors: Vec::new(),
        }
    }

    pub fn check(&mut self, program: &Program) -> AnalysisResult<Vec<String>> {
        self.errors.clear();

        match program {
            Program::Stmts(stmts) => {
                for stmt in stmts {
                    self.check_stmt(stmt);
                }
            }
        }

        if self.errors.is_empty() {
            Ok(vec![])
        } else {
            Err(AnalysisError::Message(format!("Found {} semantic errors", self.errors.len())))
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { name, init } => {
                // Check: Declarations Before Usage
                self.check_expr(init);
                self.variables.insert(name.clone(), SymbolInfo {
                    name: name.clone(),
                    declared: true,
                    used: false,
                    is_function: false,
                });
            }
            Stmt::Assign { target, value } => {
                self.check_expr(target);
                self.check_expr(value);

                // Check if target is a valid identifier
                if let Expr::Ident(name) = target {
                    if !self.variables.contains_key(name) && !self.functions.contains_key(name) {
                        self.errors.push(format!("Variable '{}' used before declaration", name));
                    }
                }

                // Check array access bounds
                self.check_array_bounds(target);
            }
            Stmt::Print { args } => {
                for arg in args {
                    self.check_expr(arg);
                }
            }
            Stmt::If { cond, then_branch, else_branch } => {
                self.check_expr(cond);
                let prev_inside_loop = self.inside_loop;
                for stmt in then_branch {
                    self.check_stmt(stmt);
                }
                if let Some(else_branch) = else_branch {
                    for stmt in else_branch {
                        self.check_stmt(stmt);
                    }
                }
                self.inside_loop = prev_inside_loop;
            }
            Stmt::While { cond, body } => {
                self.check_expr(cond);
                let prev_inside_loop = self.inside_loop;
                self.inside_loop = true;
                for stmt in body {
                    self.check_stmt(stmt);
                }
                self.inside_loop = prev_inside_loop;
            }
            Stmt::For { var, iterable, body } => {
                self.check_expr(iterable);
                let prev_inside_loop = self.inside_loop;
                self.inside_loop = true;
                // Add loop variable to scope
                self.variables.insert(var.clone(), SymbolInfo {
                    name: var.clone(),
                    declared: true,
                    used: false,
                    is_function: false,
                });
                for stmt in body {
                    self.check_stmt(stmt);
                }
                self.inside_loop = prev_inside_loop;
            }
            Stmt::Return(_) => {
                // Check: Correct Keyword Usage - return should be inside function
                if !self.inside_function {
                    self.errors.push("Return statement outside of function".to_string());
                }
            }
            Stmt::Exit => {}
            Stmt::Expr(expr) => {
                self.check_expr(expr);
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Integer(_) | Expr::Real(_) | Expr::Bool(_) | Expr::String(_) | Expr::None => {}
            Expr::Ident(name) => {
                // Check: Declarations Before Usage
                if !self.variables.contains_key(name) && !self.functions.contains_key(name) {
                    self.errors.push(format!("Variable or function '{}' used before declaration", name));
                }
            }
            Expr::Binary { left, right, .. } => {
                self.check_expr(left);
                self.check_expr(right);
            }
            Expr::Unary { expr, .. } => {
                self.check_expr(expr);
            }
            Expr::Call { callee, args } => {
                self.check_expr(callee);
                for arg in args {
                    self.check_expr(arg);
                }
            }
            Expr::Index { target, index } => {
                self.check_expr(target);
                self.check_expr(index);
                self.check_array_bounds(expr);
            }
            Expr::Member { target, .. } => {
                self.check_expr(target);
            }
            Expr::Array(elems) => {
                for elem in elems {
                    self.check_expr(elem);
                }
            }
            Expr::Tuple(elems) => {
                for elem in elems {
                    self.check_expr(&elem.value);
                }
            }
            Expr::Range(low, high) => {
                self.check_expr(low);
                self.check_expr(high);
            }
            Expr::IsType { expr, .. } => {
                self.check_expr(expr);
            }
            Expr::Func { params, body: _ } => {
                // Mark as inside function
                let prev_inside_function = self.inside_function;
                self.inside_function = true;

                // Add parameters to scope
                for param in params {
                    self.variables.insert(param.clone(), SymbolInfo {
                        name: param.clone(),
                        declared: true,
                        used: false,
                        is_function: false,
                    });
                }

                // Check body will be done later in optimizer
                self.inside_function = prev_inside_function;
            }
        }
    }

    fn check_array_bounds(&mut self, expr: &Expr) {
        if let Expr::Index { target, index } = expr {
            // Basic constant array bound checking
            if let Expr::Array(elems) = target.as_ref() {
                if let Expr::Integer(idx) = index.as_ref() {
                    if *idx < 0 || *idx >= elems.len() as i64 {
                        self.errors.push(format!("Array index {} out of bounds (array size: {})", idx, elems.len()));
                    }
                }
            }
        }
    }
}

// ============================================================================
// PART 2: OPTIMIZER (Modifies AST)
// ============================================================================

pub struct Optimizer {
    modified: bool,
}

impl Optimizer {
    pub fn new() -> Self {
        Self { modified: false }
    }

    pub fn optimize(&mut self, program: &mut Program) -> bool {
        self.modified = false;
        loop {
            let mut changed = false;
            // Run all optimizations
            changed |= self.fold_constants(program);
            changed |= self.simplify_conditionals(program);
            changed |= self.remove_unreachable_code(program);
            changed |= self.remove_unused_variables(program);

            if !changed {
                break;
            }
            self.modified = true;
        }
        self.modified
    }

    // OPTIMIZATION 1: Constant Folding
    fn fold_constants(&mut self, program: &mut Program) -> bool {
        let mut changed = false;

        match program {
            Program::Stmts(stmts) => {
                for stmt in stmts {
                    if self.fold_stmt(stmt) {
                        changed = true;
                    }
                }
            }
        }
        changed
    }

    fn fold_stmt(&mut self, stmt: &mut Stmt) -> bool {
        let mut changed = false;
        match stmt {
            Stmt::VarDecl { init, .. } => {
                if let Some(new_expr) = self.simplify_expr(init) {
                    *init = new_expr;
                    changed = true;
                }
            }
            Stmt::Assign { value, .. } => {
                if let Some(new_expr) = self.simplify_expr(value) {
                    *value = new_expr;
                    changed = true;
                }
            }
            Stmt::Print { args } => {
                for arg in args {
                    if let Some(new_expr) = self.simplify_expr(arg) {
                        *arg = new_expr;
                        changed = true;
                    }
                }
            }
            Stmt::If { cond, then_branch, else_branch } => {
                // Simplify condition
                if let Some(new_expr) = self.simplify_expr(cond) {
                    *cond = new_expr;
                    changed = true;
                }

                // Recursively optimize branches
                for s in then_branch {
                    if self.fold_stmt(s) {
                        changed = true;
                    }
                }
                if let Some(else_branch) = else_branch {
                    for s in else_branch {
                        if self.fold_stmt(s) {
                            changed = true;
                        }
                    }
                }
            }
            Stmt::While { cond, body } => {
                if let Some(new_expr) = self.simplify_expr(cond) {
                    *cond = new_expr;
                    changed = true;
                }
                for s in body {
                    if self.fold_stmt(s) {
                        changed = true;
                    }
                }
            }
            Stmt::For { iterable, body, .. } => {
                if let Some(new_expr) = self.simplify_expr(iterable) {
                    *iterable = new_expr;
                    changed = true;
                }
                for s in body {
                    if self.fold_stmt(s) {
                        changed = true;
                    }
                }
            }
            _ => {}
        }
        changed
    }

    fn simplify_expr(&mut self, expr: &mut Expr) -> Option<Expr> {
        match expr {
            Expr::Integer(_) | Expr::Real(_) | Expr::Bool(_) | Expr::String(_) | Expr::None
            | Expr::Ident(_) | Expr::Array(_) | Expr::Tuple(_) => None,

            Expr::Binary { left, op, right } => {
                // Simplify sub-expressions first
                if let Some(new_left) = self.simplify_expr(left) {
                    *left = Box::new(new_left);
                }
                if let Some(new_right) = self.simplify_expr(right) {
                    *right = Box::new(new_right);
                }

                // Now evaluate the binary expression if both sides are constants
                match (left.as_ref(), op.clone(), right.as_ref()) {
                    (Expr::Integer(a), BinOp::Add, Expr::Integer(b)) => {
                        Some(Expr::Integer(a + b))
                    }
                    (Expr::Integer(a), BinOp::Sub, Expr::Integer(b)) => {
                        Some(Expr::Integer(a - b))
                    }
                    (Expr::Integer(a), BinOp::Mul, Expr::Integer(b)) => {
                        Some(Expr::Integer(a * b))
                    }
                    (Expr::Integer(a), BinOp::Div, Expr::Integer(b)) => {
                        if *b != 0 {
                            Some(Expr::Integer(a / b))
                        } else {
                            None
                        }
                    }
                    (Expr::Integer(a), BinOp::Eq, Expr::Integer(b)) => {
                        Some(Expr::Bool(a == b))
                    }
                    (Expr::Integer(a), BinOp::Ne, Expr::Integer(b)) => {
                        Some(Expr::Bool(a != b))
                    }
                    (Expr::Integer(a), BinOp::Lt, Expr::Integer(b)) => {
                        Some(Expr::Bool(a < b))
                    }
                    (Expr::Integer(a), BinOp::Le, Expr::Integer(b)) => {
                        Some(Expr::Bool(a <= b))
                    }
                    (Expr::Integer(a), BinOp::Gt, Expr::Integer(b)) => {
                        Some(Expr::Bool(a > b))
                    }
                    (Expr::Integer(a), BinOp::Ge, Expr::Integer(b)) => {
                        Some(Expr::Bool(a >= b))
                    }
                    (Expr::Bool(a), BinOp::And, Expr::Bool(b)) => {
                        Some(Expr::Bool(*a && *b))
                    }
                    (Expr::Bool(a), BinOp::Or, Expr::Bool(b)) => {
                        Some(Expr::Bool(*a || *b))
                    }
                    (Expr::Bool(a), BinOp::Xor, Expr::Bool(b)) => {
                        Some(Expr::Bool(*a ^ *b))
                    }
                    (Expr::Real(a), BinOp::Add, Expr::Real(b)) => {
                        Some(Expr::Real(a + b))
                    }
                    (Expr::Real(a), BinOp::Sub, Expr::Real(b)) => {
                        Some(Expr::Real(a - b))
                    }
                    (Expr::Real(a), BinOp::Mul, Expr::Real(b)) => {
                        Some(Expr::Real(a * b))
                    }
                    (Expr::Real(a), BinOp::Div, Expr::Real(b)) => {
                        if *b != 0.0 {
                            Some(Expr::Real(a / b))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }

            Expr::Unary { op, expr } => {
                if let Some(new_expr) = self.simplify_expr(expr) {
                    *expr = Box::new(new_expr);
                }

                match (op.clone(), expr.as_ref()) {
                    (UnOp::Not, Expr::Bool(val)) => Some(Expr::Bool(!val)),
                    (UnOp::Neg, Expr::Integer(val)) => Some(Expr::Integer(-val)),
                    (UnOp::Neg, Expr::Real(val)) => Some(Expr::Real(-val)),
                    _ => None,
                }
            }

            _ => None,
        }
    }

    // OPTIMIZATION 2: Simplify conditionals (if true/false)
    fn simplify_conditionals(&mut self, program: &mut Program) -> bool {
        let mut changed = false;

        match program {
            Program::Stmts(stmts) => {
                for i in 0..stmts.len() {
                    if let Stmt::If { cond, then_branch, else_branch } = &stmts[i] {
                        if let Expr::Bool(true) = cond {
                            // Condition is always true - replace if with then branch
                            let then_clone = then_branch.clone();
                            *stmts = then_clone;
                            return true; // Return immediately after replacing the entire program
                        } else if let Expr::Bool(false) = cond {
                            // Condition is always false - replace with else branch or remove
                            if let Some(else_branch) = else_branch {
                                let else_clone = else_branch.clone();
                                *stmts = else_clone;
                                return true; // Return immediately after replacing the entire program
                            } else {
                                // Remove the if statement entirely
                                stmts.remove(i);
                                changed = true;
                                // Don't break, continue processing remaining statements
                            }
                        }
                    } else if let Some(ref mut stmt) = stmts.get_mut(i) {
                        if self.simplify_stmt(stmt) {
                            changed = true;
                        }
                    }
                }
            }
        }
        changed
    }

    fn simplify_stmt(&mut self, stmt: &mut Stmt) -> bool {
        match stmt {
            Stmt::If { then_branch, else_branch, .. } => {
                let mut changed = false;
                for s in then_branch {
                    if self.simplify_stmt(s) {
                        changed = true;
                    }
                }
                if let Some(else_branch) = else_branch {
                    for s in else_branch {
                        if self.simplify_stmt(s) {
                            changed = true;
                        }
                    }
                }
                changed
            }
            Stmt::While { body, .. } => {
                let mut changed = false;
                for s in body {
                    if self.simplify_stmt(s) {
                        changed = true;
                    }
                }
                changed
            }
            Stmt::For { body, .. } => {
                let mut changed = false;
                for s in body {
                    if self.simplify_stmt(s) {
                        changed = true;
                    }
                }
                changed
            }
            _ => false,
        }
    }

    // OPTIMIZATION 3: Remove unreachable code
    fn remove_unreachable_code(&mut self, program: &mut Program) -> bool {
        let mut changed = false;

        match program {
            Program::Stmts(stmts) => {
                let mut new_stmts = Vec::new();
                for stmt in stmts.iter() {
                    new_stmts.push(stmt.clone());

                    // Check if this is a return statement
                    match stmt {
                        Stmt::Return(_) | Stmt::Exit => {
                            // Everything after this is unreachable
                            break;
                        }
                        _ => {}
                    }
                }

                if new_stmts.len() != stmts.len() {
                    *stmts = new_stmts;
                    changed = true;
                }

                // Also check within if/while/for blocks
                for stmt in stmts.iter_mut() {
                    match stmt {
                        Stmt::If { then_branch, else_branch, .. } => {
                            changed |= self.remove_unreachable_code(&mut Program::Stmts(then_branch.clone()));
                            if let Some(else_branch) = else_branch {
                                changed |= self.remove_unreachable_code(&mut Program::Stmts(else_branch.clone()));
                            }
                        }
                        Stmt::While { body, .. } => {
                            changed |= self.remove_unreachable_code(&mut Program::Stmts(body.clone()));
                        }
                        Stmt::For { body, .. } => {
                            changed |= self.remove_unreachable_code(&mut Program::Stmts(body.clone()));
                        }
                        _ => {}
                    }
                }
            }
        }
        changed
    }

    // OPTIMIZATION 4: Remove unused variables
    fn remove_unused_variables(&mut self, program: &mut Program) -> bool {
        let mut changed = false;
        let mut used_vars = std::collections::HashSet::new();

        // First pass: collect all used variables
        self.collect_used_vars(program, &mut used_vars);

        // Second pass: remove unused variable declarations
        match program {
            Program::Stmts(stmts) => {
                stmts.retain(|stmt| {
                    if let Stmt::VarDecl { name, .. } = stmt {
                        if !used_vars.contains(name) {
                            changed = true;
                            return false; // Remove this declaration
                        }
                    }
                    true
                });
            }
        }
        changed
    }

    fn collect_used_vars(&self, program: &Program, used_vars: &mut std::collections::HashSet<String>) {
        match program {
            Program::Stmts(stmts) => {
                for stmt in stmts {
                    self.collect_used_vars_stmt(stmt, used_vars);
                }
            }
        }
    }

    fn collect_used_vars_stmt(&self, stmt: &Stmt, used_vars: &mut std::collections::HashSet<String>) {
        match stmt {
            Stmt::VarDecl { init, .. } => {
                self.collect_used_vars_expr(init, used_vars);
                // Note: we're collecting vars used in init, but the decl itself is being removed if unused
            }
            Stmt::Assign { target, value } => {
                self.collect_used_vars_expr(target, used_vars);
                self.collect_used_vars_expr(value, used_vars);
            }
            Stmt::Print { args } => {
                for arg in args {
                    self.collect_used_vars_expr(arg, used_vars);
                }
            }
            Stmt::If { cond, then_branch, else_branch } => {
                self.collect_used_vars_expr(cond, used_vars);
                for s in then_branch {
                    self.collect_used_vars_stmt(s, used_vars);
                }
                if let Some(else_branch) = else_branch {
                    for s in else_branch {
                        self.collect_used_vars_stmt(s, used_vars);
                    }
                }
            }
            Stmt::While { cond, body } => {
                self.collect_used_vars_expr(cond, used_vars);
                for s in body {
                    self.collect_used_vars_stmt(s, used_vars);
                }
            }
            Stmt::For { var, iterable, body } => {
                used_vars.insert(var.clone());
                self.collect_used_vars_expr(iterable, used_vars);
                for s in body {
                    self.collect_used_vars_stmt(s, used_vars);
                }
            }
            Stmt::Return(Some(expr)) => {
                self.collect_used_vars_expr(expr, used_vars);
            }
            Stmt::Expr(expr) => {
                self.collect_used_vars_expr(expr, used_vars);
            }
            _ => {}
        }
    }

    fn collect_used_vars_expr(&self, expr: &Expr, used_vars: &mut std::collections::HashSet<String>) {
        match expr {
            Expr::Ident(name) => {
                used_vars.insert(name.clone());
            }
            Expr::Binary { left, right, .. } => {
                self.collect_used_vars_expr(left, used_vars);
                self.collect_used_vars_expr(right, used_vars);
            }
            Expr::Unary { expr, .. } => {
                self.collect_used_vars_expr(expr, used_vars);
            }
            Expr::Call { callee, args } => {
                self.collect_used_vars_expr(callee, used_vars);
                for arg in args {
                    self.collect_used_vars_expr(arg, used_vars);
                }
            }
            Expr::Index { target, index } => {
                self.collect_used_vars_expr(target, used_vars);
                self.collect_used_vars_expr(index, used_vars);
            }
            Expr::Member { target, .. } => {
                self.collect_used_vars_expr(target, used_vars);
            }
            Expr::Array(elems) => {
                for elem in elems {
                    self.collect_used_vars_expr(elem, used_vars);
                }
            }
            Expr::Tuple(elems) => {
                for elem in elems {
                    self.collect_used_vars_expr(&elem.value, used_vars);
                }
            }
            Expr::Range(low, high) => {
                self.collect_used_vars_expr(low, used_vars);
                self.collect_used_vars_expr(high, used_vars);
            }
            Expr::IsType { expr, .. } => {
                self.collect_used_vars_expr(expr, used_vars);
            }
            Expr::Func { params: _, body } => {
                match body {
                    FuncBody::Expr(expr) => {
                        self.collect_used_vars_expr(expr, used_vars);
                    }
                    FuncBody::Block(stmts) => {
                        for stmt in stmts {
                            self.collect_used_vars_stmt(stmt, used_vars);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
