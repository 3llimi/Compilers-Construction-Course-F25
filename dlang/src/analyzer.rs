use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub declared: bool,
    pub used: bool,
    pub is_function: bool,
    pub symbol_type: SymbolType,  
}

#[derive(Debug, Clone)]
pub enum SymbolType {
    Variable,
    Function { param_count: usize },
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

// ====
// part 1: semantic checcks (without modifying AST)
// ====

pub struct SemanticChecker {
    scope_stack: Vec<HashMap<String, SymbolInfo>>,
    array_sizes_stack: Vec<HashMap<String, usize>>,  
    inside_function: bool,
    inside_loop: bool,
    errors: Vec<String>,
}

impl SemanticChecker {
    pub fn new() -> Self {
        Self {
            scope_stack: vec![HashMap::new()],
            array_sizes_stack: vec![HashMap::new()],
            inside_function: false,
            inside_loop: false,
            errors: Vec::new(),
        }
    }
    
    // entrance to the new scope
    fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
        self.array_sizes_stack.push(HashMap::new());
    }
    
    // exit from the scope
    fn pop_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
            self.array_sizes_stack.pop();
        }
    }
    
    fn get_symbol(&self, name: &str) -> Option<&SymbolInfo> {
        // Искать в scope_stack (не scopes!)
        for scope in self.scope_stack.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }
    
    
    fn is_declared(&self, name: &str) -> bool {
        for scope in self.scope_stack.iter().rev() {
            if scope.contains_key(name) {
                return true;
            }
        }
        false
    }
    
    fn declare_var(&mut self, name: String, info: SymbolInfo) -> bool {
        if let Some(scope) = self.scope_stack.last_mut() {
            if scope.contains_key(&name) {
                return false;  // Уже объявлена
            }
            scope.insert(name, info);
            true
        } else {
            false
        }
    }
    
    
    // arr size in curr scope
    fn record_array_size(&mut self, name: String, size: usize) {
        let current_sizes = self.array_sizes_stack.last_mut().unwrap();
        current_sizes.insert(name, size);
    }
    
    // get the size of the arr
    fn get_array_size(&self, name: &str) -> Option<usize> {
        for sizes in self.array_sizes_stack.iter().rev() {
            if let Some(&size) = sizes.get(name) {
                return Some(size);
            }
        }
        None
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
            Err(AnalysisError::Message(self.errors.join("\n")))
        }
    }
    

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { name, init } => {
                if let Expr::Func { params, .. } = init {
                    if !self.declare_var(name.clone(), SymbolInfo {
                        name: name.clone(),
                        declared: true,
                        used: false,
                        is_function: true,
                        symbol_type: SymbolType::Function {
                            param_count: params.len(),
                        },
                    }) {
                        self.errors.push(format!("Function '{}' is already declared", name));
                    }
                }
                
                // Проверить тело функции
                self.check_expr(init);
                
                if !matches!(init, Expr::Func { .. }) {
                    if !self.declare_var(name.clone(), SymbolInfo {
                        name: name.clone(),
                        declared: true,
                        used: false,
                        is_function: false,
                        symbol_type: SymbolType::Variable,
                    }) {
                        self.errors.push(format!("Variable '{}' is already declared", name));
                    }
                    
                    // Записать размер массива (если это массив)
                    if let Expr::Array(elems) = init {
                        self.record_array_size(name.clone(), elems.len());
                    }
                }
            }
            
            
            
            
            Stmt::Assign { target, value } => {
                self.check_expr(target);
                self.check_expr(value);
                
                self.check_array_bounds(target);
            }
            
            Stmt::Print { args } => {
                for arg in args {
                    self.check_expr(arg);
                }
            }
            Stmt::If { cond, then_branch, else_branch } => {
                self.check_expr(cond);
                
                // new scope for then_branch
                self.push_scope();
                for stmt in then_branch {
                    self.check_stmt(stmt);
                }
                self.pop_scope();
                
                // new scope for else_branch 
                if let Some(else_branch) = else_branch {
                    self.push_scope();
                    for stmt in else_branch {
                        self.check_stmt(stmt);
                    }
                    self.pop_scope();
                }
            }
            
            Stmt::While { cond, body } => {
                self.check_expr(cond);
                
                let prev_inside_loop = self.inside_loop;
                self.inside_loop = true;
                
                self.push_scope();
                
                for stmt in body {
                    self.check_stmt(stmt);
                }
                
                self.pop_scope();
                
                self.inside_loop = prev_inside_loop;
            }
            
            
            Stmt::For { var, iterable, body } => {
                self.check_expr(iterable);
                
                let prev_inside_loop = self.inside_loop;
                self.inside_loop = true;
                
                self.push_scope();
                
                self.declare_var(var.clone(), SymbolInfo {
                    name: var.clone(),
                    declared: true,
                    used: false,
                    is_function: false,
                    symbol_type: SymbolType::Variable,
                });
                
                for stmt in body {
                    self.check_stmt(stmt);
                }
                
                self.pop_scope();
                
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
                if !self.is_declared(name) {
                    self.errors.push(format!("Variable or function '{}' used before declaration", name));
                }
            }
            
            Expr::Binary { left, op: BinOp::Div, right } => {
                if let Expr::Integer(0) = right.as_ref() {
                    self.errors.push("Division by zero detected".to_string());
                }
                if let Expr::Real(val) = right.as_ref() {
                    if *val == 0.0 {
                        self.errors.push("Division by zero detected".to_string());
                    }
                }
                
                self.check_expr(left);
                self.check_expr(right);
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
                
                if let Expr::Ident(func_name) = callee.as_ref() {
                    if let Some(symbol) = self.get_symbol(func_name) {
                        if let SymbolType::Function { param_count } = symbol.symbol_type {
                            if args.len() != param_count {
                                self.errors.push(format!(
                                    "Function '{}' expects {} arguments, got {}",
                                    func_name,
                                    param_count,
                                    args.len()
                                ));
                            }
                        }
                    }
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
            Expr::Func { params, body } => {
                let prev_inside_function = self.inside_function;
                self.inside_function = true;
                
                self.push_scope();
                
                for param in params {
                    self.declare_var(param.clone(), SymbolInfo {
                        name: param.clone(),        
                        declared: true,             
                        used: false,                
                        is_function: false,  
                        symbol_type: SymbolType::Variable, 
                    });
                }
                
                match body {
                    FuncBody::Expr(expr) => {
                        self.check_expr(expr);
                    }
                    FuncBody::Block(stmts) => {
                        for stmt in stmts {
                            self.check_stmt(stmt);
                        }
                    }
                }

                self.pop_scope();  
                self.inside_function = prev_inside_function;
            
            }
        }
    }

    fn check_array_bounds(&mut self, expr: &Expr) {
        if let Expr::Index { target, index } = expr {
            if let Expr::Integer(idx) = index.as_ref() {
                match target.as_ref() {
                    Expr::Array(elems) => {
                        
                        if *idx < 1 || *idx > elems.len() as i64 {
                            self.errors.push(format!(
                                "Array index {} out of bounds (valid range: 1..{})", 
                                idx, elems.len()
                            ));
                        }
                    }
                    
                    Expr::Ident(name) => {
                        if let Some(size) = self.get_array_size(name) {
                            if *idx < 1 || *idx > size as i64 {
                                self.errors.push(format!(
                                    "Array index {} out of bounds (valid range: 1..{})", 
                                    idx, size
                                ));
                            }
                        }
                    }
                    
                    _ => {}
                }
            }
        }
    }
    
    
    
}

// ===
// part 2: optimizer (modifies AST)
// ===

pub struct Optimizer {
    modified: bool,
    constants: HashMap<String, Expr>,
    shadowed_vars: std::collections::HashSet<String>, 
}

impl Optimizer {
    pub fn new() -> Self {
        Self {
            modified: false,
            constants: HashMap::new(),
            shadowed_vars: std::collections::HashSet::new(), 
        }
    }

    pub fn optimize(&mut self, program: &mut Program) -> bool {
        self.modified = false;
        loop {
            let mut changed = false;
            self.constants.clear();
            self.shadowed_vars.clear();
            
            self.collect_shadowed_vars(program);
            
            // Run all optimizations
            changed |= self.collect_constants(program);      
            changed |= self.propagate_constants(program);    
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
    
    fn collect_shadowed_vars(&mut self, program: &Program) {
        match program {
            Program::Stmts(stmts) => {
                let mut outer_vars = std::collections::HashSet::new();
                
                // Собрать переменные внешнего scope
                for stmt in stmts {
                    if let Stmt::VarDecl { name, .. } = stmt {
                        outer_vars.insert(name.clone());
                    }
                }
                
                // Найти затеняемые переменные во вложенных блоках
                for stmt in stmts {
                    self.find_shadowed_in_stmt(stmt, &outer_vars);
                }
            }
        }
    }
    
    fn find_shadowed_in_stmt(&mut self, stmt: &Stmt, outer_vars: &std::collections::HashSet<String>) {
        match stmt {
            Stmt::If { then_branch, else_branch, .. } => {
                self.find_shadowed_in_block(then_branch, outer_vars);
                if let Some(else_branch) = else_branch {
                    self.find_shadowed_in_block(else_branch, outer_vars);
                }
            }
            Stmt::While { body, .. } | Stmt::For { body, .. } => {
                self.find_shadowed_in_block(body, outer_vars);
            }
            _ => {}
        }
    }
    
    fn find_shadowed_in_block(&mut self, stmts: &[Stmt], outer_vars: &std::collections::HashSet<String>) {
        for stmt in stmts {
            if let Stmt::VarDecl { name, .. } = stmt {
                // if there's variable with the same name in outer scope
                if outer_vars.contains(name) {
                    self.shadowed_vars.insert(name.clone());
                }
            }
            
            // recursively for nested blockes
            self.find_shadowed_in_stmt(stmt, outer_vars);
        }
    }

    fn collect_constants(&mut self, program: &Program) -> bool {
        match program {
            Program::Stmts(stmts) => {
                let mut assigned_vars = std::collections::HashSet::new();
                
                for stmt in stmts {
                    self.collect_assigned_vars(stmt, &mut assigned_vars);
                }
                
                for stmt in stmts {
                    if let Stmt::VarDecl { name, init } = stmt {
                        if self.is_constant_expr(init) 
                            && !assigned_vars.contains(name)
                            && !self.shadowed_vars.contains(name) {  
                            self.constants.insert(name.clone(), init.clone());
                        }
                    }
                }
            }
        }
        false
    }
    
   
    fn propagate_in_stmt(&mut self, stmt: &mut Stmt) -> bool {
        let mut changed = false;
        
        match stmt {
            Stmt::If { cond, then_branch, else_branch } => {
                if self.propagate_in_expr(cond) {
                    changed = true;
                }
                
                if !self.has_vardecl(then_branch) {
                    for s in then_branch {
                        if self.propagate_in_stmt(s) {
                            changed = true;
                        }
                    }
                }
                
                if let Some(else_branch) = else_branch {
                    if !self.has_vardecl(else_branch) {
                        for s in else_branch {
                            if self.propagate_in_stmt(s) {
                                changed = true;
                            }
                        }
                    }
                }
            }
            Stmt::While { cond, body } => {
                if self.propagate_in_expr(cond) {
                    changed = true;
                }
                
                if !self.has_vardecl(body) {
                    for s in body {
                        if self.propagate_in_stmt(s) {
                            changed = true;
                        }
                    }
                }
            }
            Stmt::For { iterable, body, .. } => {
                if self.propagate_in_expr(iterable) {
                    changed = true;
                }
                
                if !self.has_vardecl(body) {
                    for s in body {
                        if self.propagate_in_stmt(s) {
                            changed = true;
                        }
                    }
                }
            }
            Stmt::Print { args } => {
                for arg in args {
                    if self.propagate_in_expr(arg) {
                        changed = true;
                    }
                }
            }
            Stmt::Assign { value, .. } => {
                if self.propagate_in_expr(value) {
                    changed = true;
                }
            }
            _ => {}
        }
        
        changed
    }
    
    fn has_vardecl(&self, stmts: &[Stmt]) -> bool {
        stmts.iter().any(|s| matches!(s, Stmt::VarDecl { .. }))
    }
    
    
    fn collect_assigned_vars(&self, stmt: &Stmt, assigned: &mut std::collections::HashSet<String>) {
        match stmt {
            Stmt::Assign { target, .. } => {
                if let Expr::Ident(name) = target {
                    assigned.insert(name.clone());
                }
            }
            Stmt::If { then_branch, else_branch, .. } => {
                for s in then_branch {
                    self.collect_assigned_vars(s, assigned);
                }
                if let Some(else_branch) = else_branch {
                    for s in else_branch {
                        self.collect_assigned_vars(s, assigned);
                    }
                }
            }
            Stmt::While { body, .. } | Stmt::For { body, .. } => {
                for s in body {
                    self.collect_assigned_vars(s, assigned);
                }
            }
            _ => {}
        }
    }
    
    
  
    fn propagate_constants(&mut self, program: &mut Program) -> bool {
        let mut changed = false;
        
        match program {
            Program::Stmts(stmts) => {
                for stmt in stmts.iter_mut() {
                    if self.propagate_in_stmt(stmt) {
                        changed = true;
                    }
                }
            }
        }
        
        changed
    }
    
    
    
    fn propagate_in_expr(&mut self, expr: &mut Expr) -> bool {
        match expr {
            Expr::Ident(name) => {
                // if it's known constant - change
                if let Some(const_expr) = self.constants.get(name) {
                    *expr = const_expr.clone();
                    return true;
                }
            }
            Expr::Binary { left, right, .. } => {
                let mut changed = false;
                if self.propagate_in_expr(left) {
                    changed = true;
                }
                if self.propagate_in_expr(right) {
                    changed = true;
                }
                return changed;
            }
            Expr::Unary { expr: inner, .. } => {
                return self.propagate_in_expr(inner);
            }
            _ => {}
        }
        false
    }
    
    
    fn is_constant_expr(&self, expr: &Expr) -> bool {
        matches!(
            expr,
            Expr::Integer(_) | Expr::Real(_) | Expr::Bool(_) | Expr::String(_) | Expr::None
        )
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
                // sub-expressions first
                if let Some(new_left) = self.simplify_expr(left) {
                    *left = Box::new(new_left);
                }
                if let Some(new_right) = self.simplify_expr(right) {
                    *right = Box::new(new_right);
                }

                // evaluate expr (if both sides constants)
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
                            eprintln!("Warning: Division by zero detected during optimization");
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


                    
                    
                    (Expr::Ident(_), BinOp::Add, Expr::Integer(0)) => Some(*left.clone()),
                    (Expr::Integer(0), BinOp::Add, Expr::Ident(_)) => Some(*right.clone()),
                    (Expr::Ident(_), BinOp::Mul, Expr::Integer(1)) => Some(*left.clone()),
                    (Expr::Integer(1), BinOp::Mul, Expr::Ident(_)) => Some(*right.clone()),
                    (_, BinOp::Mul, Expr::Integer(0)) => Some(Expr::Integer(0)),
                    (Expr::Integer(0), BinOp::Mul, _) => Some(Expr::Integer(0)),

                    (Expr::Bool(true), BinOp::And, _) => Some(*right.clone()),
                    (_, BinOp::And, Expr::Bool(true)) => Some(*left.clone()),
                    (Expr::Bool(false), BinOp::And, _) => Some(Expr::Bool(false)),
                    (_, BinOp::And, Expr::Bool(false)) => Some(Expr::Bool(false)),
                    (Expr::Bool(true), BinOp::Or, _) => Some(Expr::Bool(true)),
                    (_, BinOp::Or, Expr::Bool(true)) => Some(Expr::Bool(true)),
                    (Expr::Bool(false), BinOp::Or, _) => Some(*right.clone()),
                    (_, BinOp::Or, Expr::Bool(false)) => Some(*left.clone()),


                    (Expr::Real(a), BinOp::Div, Expr::Real(b)) => {
                        if *b != 0.0 {
                            Some(Expr::Real(a / b))
                        } else {
                            eprintln!("Warning: Division by zero detected during optimization");
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
                let mut i = 0;
                while i < stmts.len() {
                    if let Stmt::If { cond, then_branch, else_branch } = &stmts[i] {
                        
                        let contains_vardecl = |stmts: &[Stmt]| {
                            stmts.iter().any(|s| matches!(s, Stmt::VarDecl { .. }))
                        };
                        
                        if contains_vardecl(then_branch) || 
                           else_branch.as_ref().map(|b| contains_vardecl(b)).unwrap_or(false) {
                            i += 1;
                            continue;  // skip optimization
                        }
                        
                        // safe optimization
                        if let Expr::Bool(true) = cond {
                            let then_clone = then_branch.clone();
                            stmts.splice(i..=i, then_clone);
                            changed = true;
                            continue;
                        } else if let Expr::Bool(false) = cond {
                            if let Some(else_branch) = else_branch {
                                let else_clone = else_branch.clone();
                                stmts.splice(i..=i, else_clone);
                            } else {
                                stmts.remove(i);
                            }
                            changed = true;
                            continue;
                        }
                    }
                    
                    if let Some(stmt) = stmts.get_mut(i) {
                        if self.simplify_stmt(stmt) {
                            changed = true;
                        }
                    }
                    
                    i += 1;
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
                // we're collecting vars used in init, but the decl itself is being removed if unused
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
