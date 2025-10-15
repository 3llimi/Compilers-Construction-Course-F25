#[derive (Debug, Clone, PartialEq)]
pub enum Token{
  Var, If, Then, Else, End, While, For, Loop, Func, Is,
  Exit, Return, Print, True, False, None,

  Plus, Minus, Star, Slash, Assign, Equal, NotEqual,
  Less, LessEqual, Greater, GreaterEqual,
  And, Or, Xor, Not,

  LParen, RParen, LBrace, RBrace, LBracket, RBracket,
  Comma, Semicolon, Dot, In, Range, Arrow, Newline,

  // keywords of types for operator is
  TypeInt,     
  TypeReal,    
  TypeBool,    
  TypeString,  

  Identifier(String),
  Integer(i64),
  Real(f64),
  String(String),
  Comment(String),
  Error {
    message: String,
    line: usize,
    col: usize,
  },

  EOF,
}
