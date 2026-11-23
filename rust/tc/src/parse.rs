//! Trice syntax (docs/trice.md sections 5 to 7) into the tree of ast.rs.

use crate::Error;
use crate::ast::*;
use crate::lex::{Tok, Token};

pub struct Parser<'a> {
    toks: Vec<Token>,
    pos: usize,
    file: &'a str,
}

const TYPE_WORDS: [&str; 6] = ["int", "tryte", "trit", "bool", "void", "struct"];

/// Binary operators by precedence, loosest first (trice.md section 5).
const LEVELS: [&[&str]; 10] = [
    &["||"],
    &["&&"],
    &["|"],
    &["&"],
    &["==", "!="],
    &["<", "<=", ">", ">="],
    &["<=>"],
    &["<<", ">>"],
    &["+", "-"],
    &["*", "/", "%"],
];

const ASSIGN: [&str; 10] = ["=", "+=", "-=", "*=", "/=", "%=", "<<=", ">>=", "&=", "|="];

impl<'a> Parser<'a> {
    pub fn new(toks: Vec<Token>, file: &'a str) -> Self {
        Parser { toks, pos: 0, file }
    }

    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }

    fn peek_at(&self, n: usize) -> &Tok {
        &self.toks[(self.pos + n).min(self.toks.len() - 1)].tok
    }

    fn line(&self) -> usize {
        self.toks[self.pos].line
    }

    fn next(&mut self) -> Tok {
        let t = self.toks[self.pos].tok.clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn err<T>(&self, msg: impl Into<String>) -> Result<T, Error> {
        Err(Error::new(self.file, self.line(), msg.into()))
    }

    fn is_punct(&self, p: &str) -> bool {
        matches!(self.peek(), Tok::Punct(q) if *q == p)
    }

    fn is_word(&self, w: &str) -> bool {
        matches!(self.peek(), Tok::Ident(q) if q == w)
    }

    fn eat(&mut self, p: &str) -> bool {
        if self.is_punct(p) {
            self.next();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, p: &str) -> Result<(), Error> {
        if self.eat(p) {
            Ok(())
        } else {
            self.err(format!("expected {p}, found {}", describe(self.peek())))
        }
    }

    fn ident(&mut self) -> Result<String, Error> {
        match self.peek().clone() {
            Tok::Ident(name) if !is_keyword(&name) => {
                self.next();
                Ok(name)
            }
            other => self.err(format!("expected a name, found {}", describe(&other))),
        }
    }

    fn starts_type(&self) -> bool {
        matches!(self.peek(), Tok::Ident(w) if TYPE_WORDS.contains(&w.as_str()))
    }

    fn base_type(&mut self) -> Result<Type, Error> {
        let t = match self.peek() {
            Tok::Ident(w) => match w.as_str() {
                "int" => Type::Int,
                "tryte" => Type::Tryte,
                "trit" => Type::Trit,
                "bool" => Type::Bool,
                "void" => Type::Void,
                "struct" => {
                    self.next();
                    return Ok(Type::Struct(self.ident()?));
                }
                _ => return self.err(format!("expected a type, found {w}")),
            },
            other => return self.err(format!("expected a type, found {}", describe(other))),
        };
        self.next();
        Ok(t)
    }

    /// A type: a base type and its stars.
    fn ty(&mut self) -> Result<Type, Error> {
        let mut t = self.base_type()?;
        while self.eat("*") {
            t = Type::Ptr(Box::new(t));
        }
        Ok(t)
    }

    pub fn program(&mut self) -> Result<Vec<Item>, Error> {
        let mut items = Vec::new();
        while *self.peek() != Tok::Eof {
            items.push(self.item()?);
        }
        Ok(items)
    }

    fn item(&mut self) -> Result<Item, Error> {
        let line = self.line();
        if self.is_word("import") {
            self.next();
            let Tok::Str(path) = self.next() else {
                return self.err("import needs a path in quotes");
            };
            self.expect(";")?;
            let path: String = path.iter().filter_map(|&c| char::from_u32(c as u32)).collect();
            return Ok(Item::Import(path, line));
        }
        if self.is_word("const") {
            self.next();
            let ty = self.ty()?;
            let name = self.ident()?;
            self.expect("=")?;
            let value = self.expr()?;
            self.expect(";")?;
            return Ok(Item::Const { ty, name, value, line });
        }
        if self.is_word("struct")
            && matches!(self.peek_at(1), Tok::Ident(_))
            && matches!(self.peek_at(2), Tok::Punct("{"))
        {
            self.next();
            let name = self.ident()?;
            self.expect("{")?;
            let mut fields = Vec::new();
            while !self.eat("}") {
                if *self.peek() == Tok::Eof {
                    return self.err("struct not closed");
                }
                let ty = self.ty()?;
                let field = self.ident()?;
                let array = if self.eat("[") {
                    let n = self.expr()?;
                    self.expect("]")?;
                    Some(n)
                } else {
                    None
                };
                self.expect(";")?;
                fields.push((ty, field, array));
            }
            self.expect(";")?;
            return Ok(Item::Struct { name, fields, line });
        }
        let external = self.is_word("extern");
        if external {
            self.next();
        }
        let ty = self.ty()?;
        let name = self.ident()?;
        if self.eat("(") {
            let mut params = Vec::new();
            if !self.is_punct(")") {
                if self.is_word("void") && matches!(self.peek_at(1), Tok::Punct(")")) {
                    self.next();
                } else {
                    loop {
                        let pt = self.ty()?;
                        let pn = self.ident()?;
                        if self.is_punct("[") {
                            return self.err("an array parameter is written as a pointer");
                        }
                        params.push((pt, pn));
                        if !self.eat(",") {
                            break;
                        }
                    }
                }
            }
            self.expect(")")?;
            let body = if external {
                self.expect(";")?;
                None
            } else {
                Some(self.block()?)
            };
            return Ok(Item::Func(Func { ret: ty, name, params, body, line }));
        }
        if external {
            return self.err("extern declares functions only");
        }
        let (array, init) = self.declarator_rest()?;
        Ok(Item::Global { ty, name, array, init, line })
    }

    /// After a variable's name: `[n]`, `= init`, then `;`.
    fn declarator_rest(&mut self) -> Result<(Option<Expr>, Option<Expr>), Error> {
        let array = if self.eat("[") {
            let n = self.expr()?;
            self.expect("]")?;
            Some(n)
        } else {
            None
        };
        let init = if self.eat("=") { Some(self.initialiser()?) } else { None };
        self.expect(";")?;
        Ok((array, init))
    }

    /// An expression, or a list of initialisers in braces.
    fn initialiser(&mut self) -> Result<Expr, Error> {
        let line = self.line();
        if !self.eat("{") {
            return self.expr();
        }
        let mut items = Vec::new();
        while !self.eat("}") {
            items.push(self.initialiser()?);
            if !self.eat(",") {
                self.expect("}")?;
                break;
            }
        }
        Ok(Expr { kind: ExprKind::List(items), line })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, Error> {
        self.expect("{")?;
        let mut body = Vec::new();
        while !self.eat("}") {
            if *self.peek() == Tok::Eof {
                return self.err("block not closed");
            }
            body.push(self.stmt()?);
        }
        Ok(body)
    }

    fn stmt(&mut self) -> Result<Stmt, Error> {
        let line = self.line();
        if let Tok::Asm(text) = self.peek().clone() {
            self.next();
            return Ok(Stmt::Asm(text, line));
        }
        if self.is_punct("{") {
            return Ok(Stmt::Block(self.block()?));
        }
        if self.starts_type() {
            let ty = self.ty()?;
            let name = self.ident()?;
            let (array, init) = self.declarator_rest()?;
            return Ok(Stmt::Local { ty, name, array, init, line });
        }
        let word = match self.peek() {
            Tok::Ident(w) => w.clone(),
            _ => String::new(),
        };
        match word.as_str() {
            "if" => {
                self.next();
                let cond = self.paren_expr()?;
                let then = Box::new(self.stmt()?);
                let other = if self.is_word("else") {
                    self.next();
                    Some(Box::new(self.stmt()?))
                } else {
                    None
                };
                Ok(Stmt::If(cond, then, other))
            }
            "while" => {
                self.next();
                let cond = self.paren_expr()?;
                Ok(Stmt::While(cond, Box::new(self.stmt()?)))
            }
            "for" => {
                self.next();
                self.expect("(")?;
                let init = if self.eat(";") {
                    None
                } else if self.starts_type() {
                    Some(Box::new(self.stmt()?))
                } else {
                    let e = self.expr()?;
                    self.expect(";")?;
                    Some(Box::new(Stmt::Expr(e)))
                };
                let cond = if self.is_punct(";") { None } else { Some(self.expr()?) };
                self.expect(";")?;
                let step = if self.is_punct(")") { None } else { Some(self.expr()?) };
                self.expect(")")?;
                Ok(Stmt::For(init, cond, step, Box::new(self.stmt()?)))
            }
            "break" | "continue" => {
                self.next();
                self.expect(";")?;
                Ok(if word == "break" { Stmt::Break(line) } else { Stmt::Continue(line) })
            }
            "return" => {
                self.next();
                let value = if self.is_punct(";") { None } else { Some(self.expr()?) };
                self.expect(";")?;
                Ok(Stmt::Return(value, line))
            }
            "which" => self.which(),
            "else" => self.err("else without if"),
            _ => {
                let e = self.expr()?;
                self.expect(";")?;
                Ok(Stmt::Expr(e))
            }
        }
    }

    fn which(&mut self) -> Result<Stmt, Error> {
        self.next();
        let value = self.paren_expr()?;
        self.expect("{")?;
        let mut arms: Vec<(Arm, Vec<Stmt>, usize)> = Vec::new();
        let mut kinds = Vec::new();
        while !self.eat("}") {
            let line = self.line();
            let Some((arm, is_bool)) = self.arm_label() else {
                return self.err("expected an arm: <, =, > or true, unknown, false");
            };
            self.expect(":")?;
            if arms.iter().any(|(a, _, _)| *a == arm) {
                return self.err("this case already has an arm");
            }
            kinds.push(is_bool);
            let mut body = Vec::new();
            while self.arm_label_ahead().is_none() && !self.is_punct("}") {
                if *self.peek() == Tok::Eof {
                    return self.err("which not closed");
                }
                body.push(self.stmt()?);
            }
            arms.push((arm, body, line));
        }
        let bool_arms = kinds.first().copied().unwrap_or(false);
        if kinds.iter().any(|k| *k != bool_arms) {
            return self.err("which mixes < = > arms with true, unknown, false arms");
        }
        Ok(Stmt::Which { value, arms, bool_arms })
    }

    fn arm_label_ahead(&self) -> Option<(Arm, bool)> {
        if !matches!(self.peek_at(1), Tok::Punct(":")) {
            return None;
        }
        match self.peek() {
            Tok::Punct("<") => Some((Arm::Neg, false)),
            Tok::Punct("=") => Some((Arm::Zero, false)),
            Tok::Punct(">") => Some((Arm::Pos, false)),
            Tok::Ident(w) if w == "false" => Some((Arm::Neg, true)),
            Tok::Ident(w) if w == "unknown" => Some((Arm::Zero, true)),
            Tok::Ident(w) if w == "true" => Some((Arm::Pos, true)),
            _ => None,
        }
    }

    fn arm_label(&mut self) -> Option<(Arm, bool)> {
        let found = self.arm_label_ahead();
        if found.is_some() {
            self.next();
        }
        found
    }

    fn paren_expr(&mut self) -> Result<Expr, Error> {
        self.expect("(")?;
        let e = self.expr()?;
        self.expect(")")?;
        Ok(e)
    }

    pub fn expr(&mut self) -> Result<Expr, Error> {
        let lhs = self.conditional()?;
        let line = self.line();
        if let Tok::Punct(p) = self.peek()
            && ASSIGN.contains(p)
        {
            let op = *p;
            self.next();
            let rhs = self.expr()?;
            return Ok(Expr { kind: ExprKind::Assign(op, Box::new(lhs), Box::new(rhs)), line });
        }
        Ok(lhs)
    }

    fn conditional(&mut self) -> Result<Expr, Error> {
        let cond = self.binary(0)?;
        let line = self.line();
        if self.eat("?") {
            let a = self.expr()?;
            self.expect(":")?;
            let b = self.conditional()?;
            return Ok(Expr {
                kind: ExprKind::Cond(Box::new(cond), Box::new(a), Box::new(b)),
                line,
            });
        }
        Ok(cond)
    }

    fn binary(&mut self, level: usize) -> Result<Expr, Error> {
        if level == LEVELS.len() {
            return self.unary();
        }
        let mut lhs = self.binary(level + 1)?;
        loop {
            let line = self.line();
            let op = match self.peek() {
                Tok::Punct(p) if LEVELS[level].contains(p) => *p,
                Tok::Punct("^") if level == 2 => {
                    return self.err("Trice has no ^: use tsum(a, b) or tmul(a, b)");
                }
                _ => return Ok(lhs),
            };
            self.next();
            let rhs = self.binary(level + 1)?;
            lhs = Expr { kind: ExprKind::Binary(op, Box::new(lhs), Box::new(rhs)), line };
        }
    }

    fn unary(&mut self) -> Result<Expr, Error> {
        let line = self.line();
        let op = match self.peek() {
            Tok::Punct(p @ ("-" | "!" | "*" | "&" | "++" | "--")) => *p,
            Tok::Punct("~") => {
                return self.err("Trice has no ~: -x already flips every trit");
            }
            Tok::Punct("+") => {
                self.next();
                return self.unary();
            }
            Tok::Punct("(") if matches!(self.peek_at(1), Tok::Ident(w) if TYPE_WORDS.contains(&w.as_str())) =>
            {
                self.next();
                let t = self.ty()?;
                self.expect(")")?;
                let e = self.unary()?;
                return Ok(Expr { kind: ExprKind::Cast(t, Box::new(e)), line });
            }
            Tok::Ident(w) if w == "sizeof" => {
                self.next();
                if self.is_punct("(")
                    && matches!(self.peek_at(1), Tok::Ident(w) if TYPE_WORDS.contains(&w.as_str()))
                {
                    self.next();
                    let t = self.ty()?;
                    self.expect(")")?;
                    return Ok(Expr { kind: ExprKind::SizeofType(t), line });
                }
                let e = self.unary()?;
                return Ok(Expr { kind: ExprKind::SizeofExpr(Box::new(e)), line });
            }
            _ => return self.postfix(),
        };
        self.next();
        let operand = Box::new(self.unary()?);
        let kind = match op {
            "++" | "--" => ExprKind::Step { op, post: false, target: operand },
            _ => ExprKind::Unary(op, operand),
        };
        Ok(Expr { kind, line })
    }

    fn postfix(&mut self) -> Result<Expr, Error> {
        let mut e = self.primary()?;
        loop {
            let line = self.line();
            if self.eat("[") {
                let i = self.expr()?;
                self.expect("]")?;
                e = Expr { kind: ExprKind::Index(Box::new(e), Box::new(i)), line };
            } else if self.is_punct("++") || self.is_punct("--") {
                let Tok::Punct(op) = self.next() else { unreachable!() };
                e = Expr { kind: ExprKind::Step { op, post: true, target: Box::new(e) }, line };
            } else if self.is_punct(".") || self.is_punct("->") {
                let arrow = self.is_punct("->");
                self.next();
                let field = self.ident()?;
                e = Expr { kind: ExprKind::Field(Box::new(e), field, arrow), line };
            } else {
                return Ok(e);
            }
        }
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        let line = self.line();
        let kind = match self.next() {
            Tok::Int(v) => ExprKind::Int(v),
            Tok::Str(s) => ExprKind::Str(s),
            Tok::Ident(w) if w == "true" => ExprKind::Bool(1),
            Tok::Ident(w) if w == "false" => ExprKind::Bool(-1),
            Tok::Ident(w) if w == "unknown" => ExprKind::Bool(0),
            Tok::Ident(w) if !is_keyword(&w) => {
                if self.eat("(") {
                    let mut args = Vec::new();
                    if !self.eat(")") {
                        loop {
                            args.push(self.expr()?);
                            if self.eat(")") {
                                break;
                            }
                            self.expect(",")?;
                        }
                    }
                    ExprKind::Call(w, args)
                } else {
                    ExprKind::Name(w)
                }
            }
            Tok::Punct("(") => {
                let e = self.expr()?;
                self.expect(")")?;
                return Ok(e);
            }
            other => {
                let msg = format!("expected an expression, found {}", describe(&other));
                return Err(Error::new(self.file, line, msg));
            }
        };
        Ok(Expr { kind, line })
    }
}

pub fn is_keyword(w: &str) -> bool {
    matches!(
        w,
        "bool"
            | "break"
            | "const"
            | "continue"
            | "else"
            | "extern"
            | "false"
            | "for"
            | "if"
            | "import"
            | "int"
            | "return"
            | "sizeof"
            | "struct"
            | "trit"
            | "true"
            | "tryte"
            | "unknown"
            | "void"
            | "which"
            | "while"
            | "asm"
    )
}

fn describe(t: &Tok) -> String {
    match t {
        Tok::Ident(w) => w.clone(),
        Tok::Int(v) => v.to_string(),
        Tok::Str(_) => "a string".into(),
        Tok::Punct(p) => p.to_string(),
        Tok::Asm(_) => "an asm block".into(),
        Tok::Eof => "the end of the file".into(),
    }
}
