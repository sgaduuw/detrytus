//! The parsed program.

use std::cell::RefCell;
use std::collections::HashMap;

/// A struct's layout (abi.md 1.2): its fields with their offsets from
/// the struct's address, its size, and whether it is word-aligned.
#[derive(Debug, Clone)]
pub struct Layout {
    pub fields: Vec<(String, Type, i128)>,
    pub size: i128,
    pub aligned: bool,
}

thread_local! {
    /// The layouts of the program being compiled, by struct name. Types
    /// name their struct, so that a struct can point to its own kind;
    /// their sizes are looked up here.
    static STRUCTS: RefCell<HashMap<String, Layout>> = RefCell::new(HashMap::new());
}

pub fn clear_structs() {
    STRUCTS.with(|s| s.borrow_mut().clear());
}

pub fn define_struct(name: &str, layout: Layout) {
    STRUCTS.with(|s| s.borrow_mut().insert(name.to_string(), layout));
}

/// A defined struct's layout; None while it is only named.
pub fn layout(name: &str) -> Option<Layout> {
    STRUCTS.with(|s| s.borrow().get(name).cloned())
}

/// Lay out fields in order by abi.md 1.2.
pub fn lay_out(fields: Vec<(String, Type)>) -> Layout {
    let aligned = fields.iter().any(|(_, t)| t.word_aligned());
    let start = if aligned { -1 } else { 0 };
    let mut cursor: i128 = start;
    let mut out = Vec::new();
    for (name, ty) in fields {
        let off = if ty.word_aligned() {
            // The first multiple of 3 whose tryte below is free.
            let mut m = cursor + 1;
            m += (3 - m.rem_euclid(3)) % 3;
            cursor = m - 1 + ty.size();
            m
        } else {
            let at = cursor;
            cursor += ty.size();
            at
        };
        out.push((name, ty, off));
    }
    let mut size = cursor - start;
    if aligned {
        size += (3 - size % 3) % 3;
    }
    Layout { fields: out, size, aligned }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Tryte,
    Trit,
    Bool,
    Void,
    Ptr(Box<Type>),
    Array(Box<Type>, i128),
    /// A struct, by name: its layout is in the registry above.
    Struct(String),
}

impl Type {
    /// Trytes the type occupies (abi.md 1.1).
    pub fn size(&self) -> i128 {
        match self {
            Type::Int | Type::Ptr(_) => 3,
            Type::Tryte | Type::Trit | Type::Bool => 1,
            Type::Void => 0,
            Type::Array(t, n) => t.size() * n,
            Type::Struct(n) => layout(n).map_or(0, |l| l.size),
        }
    }

    /// Word-aligned: centred on its address (abi.md 1.2).
    pub fn word_aligned(&self) -> bool {
        match self {
            Type::Int | Type::Ptr(_) => true,
            Type::Array(t, _) => t.word_aligned(),
            Type::Struct(n) => layout(n).is_some_and(|l| l.aligned),
            _ => false,
        }
    }

    /// A number in arithmetic: `int`, `tryte` or `trit`.
    pub fn numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Tryte | Type::Trit)
    }

    /// An array or a struct: its value is its address.
    pub fn aggregate(&self) -> bool {
        matches!(self, Type::Array(..) | Type::Struct(_))
    }

    /// Complete: a struct is defined, and an array's elements are.
    pub fn complete(&self) -> bool {
        match self {
            Type::Struct(n) => layout(n).is_some(),
            Type::Array(t, _) => t.complete(),
            _ => true,
        }
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
            Type::Struct(n) => format!("struct {n}"),
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
    /// `s.f` (`arrow` false) or `p->f` (true).
    Field(Box<Expr>, String, bool),
    /// `{ a, b, ... }`, in an initialiser only.
    List(Vec<Expr>),
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
    Const {
        ty: Type,
        name: String,
        value: Expr,
        line: usize,
    },
    Global {
        ty: Type,
        name: String,
        array: Option<Expr>,
        init: Option<Expr>,
        line: usize,
    },
    /// `struct S { ... };`: each field's type, name and array length.
    Struct {
        name: String,
        fields: Vec<(Type, String, Option<Expr>)>,
        line: usize,
    },
    Func(Func),
}
