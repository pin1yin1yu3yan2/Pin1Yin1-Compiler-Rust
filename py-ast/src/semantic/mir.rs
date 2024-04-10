use super::{declare::*, mangle::Mangler};
use crate::{benches, ops::Operators};
use std::borrow::Cow;

pub use py_ir::ir::{ComplexType, PrimitiveType, TypeDefine};

pub type Statements = Vec<Statement>;

#[derive(Debug, Clone)]
pub enum Statement {
    FnDefine(FnDefine),
    Compute(Compute),
    VarDefine(VarDefine),
    VarStore(VarStore),
    FnCall(FnCall),
    Block(Statements),
    If(If),
    While(While),
    Return(Return),
}

mod from_impls {
    use super::*;
    impl From<FnDefine> for Statement {
        fn from(v: FnDefine) -> Self {
            Self::FnDefine(v)
        }
    }

    impl From<Compute> for Statement {
        fn from(v: Compute) -> Self {
            Self::Compute(v)
        }
    }

    impl From<VarDefine> for Statement {
        fn from(v: VarDefine) -> Self {
            Self::VarDefine(v)
        }
    }

    impl From<VarStore> for Statement {
        fn from(v: VarStore) -> Self {
            Self::VarStore(v)
        }
    }

    impl From<FnCall> for Statement {
        fn from(v: FnCall) -> Self {
            Self::FnCall(v)
        }
    }

    impl From<Statements> for Statement {
        fn from(v: Statements) -> Self {
            Self::Block(v)
        }
    }

    impl From<If> for Statement {
        fn from(v: If) -> Self {
            Self::If(v)
        }
    }

    impl From<While> for Statement {
        fn from(v: While) -> Self {
            Self::While(v)
        }
    }

    impl From<Return> for Statement {
        fn from(v: Return) -> Self {
            Self::Return(v)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    mangle: Cow<'static, str>,
}

impl Type {
    const DEFAULT_INT: Self = Self {
        mangle: Cow::Borrowed("i64"),
    };
    const DEFAULT_FLOAT: Self = Self {
        mangle: Cow::Borrowed("f32"),
    };
}

#[derive(Debug, Clone)]
pub enum AtomicExpr {
    Char(char),
    String(String),
    Integer(usize),
    Float(f64),
    Variable(String),
    FnCall(FnCall),
    // #[deprecated = "unsupported now"]
    // Initialization(Vec<Expr>),
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub val: AtomicExpr,
    pub ty: GroupIdx,
}

impl Variable {
    pub fn new(val: AtomicExpr, ty: GroupIdx) -> Self {
        Variable { val, ty }
    }

    pub fn is_literal(&self) -> bool {
        !matches!(self.val, AtomicExpr::FnCall(..) | AtomicExpr::Variable(..))
    }

    pub fn literal_benches<M: Mangler>(atomic: &AtomicExpr) -> Vec<BenchBuilder<M>> {
        match atomic {
            AtomicExpr::Char(_) => benches! {() => PrimitiveType::char()},
            // String: greatly in processing...
            AtomicExpr::String(_) => benches! {() => ComplexType::string()},
            AtomicExpr::Integer(_) => benches! {
                () => PrimitiveType::U8, () => PrimitiveType::U16,
                () => PrimitiveType::U32,() => PrimitiveType::U64,
                () => PrimitiveType::U128,() => PrimitiveType::Usize,
                () => PrimitiveType::I8, () => PrimitiveType::I16,
                () => PrimitiveType::I32,() => PrimitiveType::I64,
                () => PrimitiveType::I128,() => PrimitiveType::Isize

            },
            AtomicExpr::Float(_) => benches! {
                () => PrimitiveType::F32,
                () => PrimitiveType::F64
            },

            _ => unreachable!("should be filtered out before, and extend stored by GroupIdx"),
        }
    }
}

pub type Variables = Vec<Variable>;

/// be different of [py_ir::ir::OperateExpr], this is **not** around primitive types
///
/// even the function overload may be delay

#[derive(Debug, Clone)]
pub enum OperateExpr {
    // type must be known, and then pick a operator-overload
    Unary(Operators, Variable),
    Binary(Operators, Variable, Variable),
}

impl OperateExpr {
    pub fn binary(op: Operators, l: impl Into<Variable>, r: impl Into<Variable>) -> Self {
        Self::Binary(op, l.into(), r.into())
    }

    pub fn unary(op: Operators, v: impl Into<Variable>) -> Self {
        Self::Unary(op, v.into())
    }
}

#[derive(Debug, Clone)]
pub struct Compute {
    pub ty: PrimitiveType,
    pub name: String,
    pub eval: OperateExpr,
}

#[derive(Debug, Clone)]
pub enum TypeDecorators {
    Const,
    Array,
    Reference,
    Pointer,
    SizedArray(usize),
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub ty: TypeDefine,
    /// using string because its the name of parameter, not a value
    pub name: String,
}

pub type Parameters = Vec<Parameter>;

#[derive(Debug, Clone)]
pub struct FnDefine {
    pub ty: TypeDefine,
    pub name: String,
    pub params: Parameters,
    pub body: Statements,
}

#[derive(Debug, Clone)]
pub struct VarDefine {
    pub ty: TypeDefine,
    pub name: String,
    pub init: Option<Variable>,
}

#[derive(Debug, Clone)]
pub struct VarStore {
    pub name: String,
    pub val: Variable,
}

#[derive(Debug, Clone)]
pub struct FnCall {
    pub args: Variables,
}

#[derive(Debug, Clone)]
pub struct Condition {
    // the final value of the condition
    pub val: Variable,
    pub compute: Statements,
}

#[derive(Debug, Clone)]
pub struct IfBranch {
    pub cond: Condition,
    pub body: Statements,
}

#[derive(Debug, Clone)]
pub struct If {
    pub branches: Vec<IfBranch>,
    pub else_: Option<Statements>,
}

#[derive(Debug, Clone)]
pub struct While {
    pub cond: Condition,
    pub body: Statements,
}

#[derive(Debug, Clone)]
pub struct Return {
    pub val: Option<Variable>,
}
