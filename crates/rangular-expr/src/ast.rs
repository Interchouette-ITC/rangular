#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Lit(Literal),
    Ident(String),
    Unary {
        op: UnOp,
        expr: Box<Self>,
    },
    Binary {
        op: BinOp,
        left: Box<Self>,
        right: Box<Self>,
    },
    Call {
        callee: Box<Self>,
        args: Vec<Self>,
    },
    Ternary {
        cond: Box<Self>,
        then_branch: Box<Self>,
        else_branch: Box<Self>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Str(String),
    Num(f64),
    Bool(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Or,
    And,
    Eq,
    Ne,
    Add,
}
