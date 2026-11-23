//! The parsed program.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Tryte,
    Trit,
    Bool,
    Void,
    Ptr(Box<Type>),
    Array(Box<Type>, i128),
}

impl Type {
    /// Trytes the type occupies (abi.md 1.1).
    pub fn size(&self) -> i128 {
        match self {
            Type::Int | Type::Ptr(_) => 3,
            Type::Tryte | Type::Trit | Type::Bool => 1,
            Type::Void => 0,
            Type::Array(t, n) => t.size() * n,
        }
    }

    /// Word-aligned: centred on its address (abi.md 1.2).
    pub fn word_aligned(&self) -> bool {
        match self {
            Type::Int | Type::Ptr(_) => true,
            Type::Array(t, _) => t.word_aligned(),
            _ => false,
        }
    }

    /// A number in arithmetic: `int`, `tryte` or `trit`.
    pub fn numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Tryte | Type::Trit)
    }

    pub fn is_ptr(&self) -> bool {
        matches!(self, Type::Ptr(_))
    }

    /// The value an expression of this type gives: arrays become
    /// pointers to their first element.
    pub fn decay(&self) -> Type {
        match self {
            Type::Array(t, _) => Type::Ptr(t.clone()),
            t => t.clone(),
        }
    }

    /// The name with its article, for messages: "an int", "a tryte*".
    pub fn a(&self) -> String {
        let n = self.name();
        if n.starts_with('i') { format!("an {n}") } else { format!("a {n}") }
    }

    pub fn name(&self) -> String {
        match self {
            Type::Int => "int".into(),
            Type::Tryte => "tryte".into(),
            Type::Trit => "trit".into(),
            Type::Bool => "bool".into(),
            Type::Void => "void".into(),
            Type::Ptr(t) => format!("{}*", t.name()),
            Type::Array(t, n) => format!("{}[{n}]", t.name()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Int(i128),
    Str(Vec<i128>),
    Bool(i128),
    Name(String),
    Unary(&'static str, Box<Expr>),
    Binary(&'static str, Box<Expr>, Box<Expr>),
    /// `=` or a compound operator such as `+=`.
    Assign(&'static str, Box<Expr>, Box<Expr>),
    Cond(Box<Expr>, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Cast(Type, Box<Expr>),
    SizeofType(Type),
    SizeofExpr(Box<Expr>),
    /// `++x` or `--x`: `post` false; `x++` or `x--`: true.
    Step {
        op: &'static str,
        post: bool,
        target: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arm {
    Neg,
    Zero,
    Pos,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    /// `array` is the element count of an array, still to be evaluated.
    Local {
        ty: Type,
        name: String,
        array: Option<Expr>,
        init: Option<Expr>,
        line: usize,
    },
    Expr(Expr),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    For(Option<Box<Stmt>>, Option<Expr>, Option<Expr>, Box<Stmt>),
    Break(usize),
    Continue(usize),
    Return(Option<Expr>, usize),
    /// The value and its arms, each named by the case it handles;
    /// `bool` arms say whether they were written as true/unknown/false.
    Which {
        value: Expr,
        arms: Vec<(Arm, Vec<Stmt>, usize)>,
        bool_arms: bool,
    },
    Asm(String, usize),
}

#[derive(Debug, Clone)]
pub struct Func {
    pub ret: Type,
    pub name: String,
    pub params: Vec<(Type, String)>,
    /// None for `extern`.
    pub body: Option<Vec<Stmt>>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum Item {
    Import(String, usize),
    Const { ty: Type, name: String, value: Expr, line: usize },
    Global { ty: Type, name: String, array: Option<Expr>, init: Option<Expr>, line: usize },
    Func(Func),
}
