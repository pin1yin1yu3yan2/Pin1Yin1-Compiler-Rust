use pin1yin1_parser::{ParseUnit, PU};

use crate::keywords::operators::Operators;

pub type Statements = Vec<Statement>;

#[derive(Debug, Clone)]
pub enum Statement {
    FnDef(FuncionDefinition),
    VarDef(VariableDefinition),
    VarAss(VariableAssign),
    FnCall(FunctionCall),
    CodeBlock(Statements),
    If(If),
    While(While),
    Return(Return),
    // comment
    Empty,
}

#[derive(Debug, Clone)]
pub struct FuncionDefinition {
    pub type_: TypeDefinition,
    pub name: String,
    pub args: Parameters,
    pub body: Statements,
}

#[derive(Debug, Clone)]
pub struct VariableDefinition {
    pub type_: TypeDefinition,
    pub name: String,
    pub init: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct VariableAssign {
    pub name: String,
    pub assign: Expr,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub args: Arguments,
}

#[derive(Debug, Clone)]
pub struct IfBranch {
    pub conds: Vec<Expr>,
    pub body: Statements,
}

#[derive(Debug, Clone)]
pub struct If {
    pub branches: Vec<IfBranch>,
    pub else_: Option<Statements>,
}

#[derive(Debug, Clone)]
pub struct While {
    pub conds: Vec<Expr>,
    pub body: Statements,
}

#[derive(Debug, Clone)]
pub struct Return {
    pub val: Option<Expr>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Char(char),
    String(String),
    Digit(usize),
    Float(f64),
    Variable(String),
    FuncionCall(FunctionCall),
    Unary(Operators, Box<Expr>),
    Binary(Operators, Box<Expr>, Box<Expr>),
    Initialization(Vec<Expr>),
}

pub type Parameters = Vec<Parameter>;

#[derive(Debug, Clone)]
pub struct Parameter {
    pub type_: TypeDefinition,
    pub name: String,
}

pub type Arguments = Vec<Argument>;

pub type Argument = Expr;

#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub const_: bool,
    pub decorators: Vec<TypeDecorators>,
    pub width: Option<usize>,
    pub sign: Option<bool>,
    pub type_: String,
}

#[derive(Debug, Clone)]
pub enum TypeDecorators {
    Array,
    Reference,
    Pointer,
    SizedArray(usize),
}

#[cfg(feature = "ser")]
mod _serde {
    use super::*;
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = TypeDecorators;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("Array, Reference, Pointer or a digit")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match v {
                "Array" => Ok(TypeDecorators::Array),
                "Reference" => Ok(TypeDecorators::Reference),
                "Pointer" => Ok(TypeDecorators::Pointer),
                _ => v
                    .parse::<usize>()
                    .map(TypeDecorators::SizedArray)
                    .map_err(E::custom),
            }
        }
    }

    impl serde::Serialize for TypeDecorators {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                TypeDecorators::Array => serializer.serialize_str("Array"),
                TypeDecorators::Reference => serializer.serialize_str("Reference"),
                TypeDecorators::Pointer => serializer.serialize_str("Pointer"),
                TypeDecorators::SizedArray(v) => serializer.serialize_str(&format!("{v}")),
            }
        }
    }

    impl<'de> serde::Deserialize<'de> for TypeDecorators {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_str(Visitor)
        }
    }
}

fn take_all<'s, P: ParseUnit, R>(iter: impl IntoIterator<Item = PU<'s, P>>) -> Vec<R>
where
    R: From<P::Target<'s>>,
{
    iter.into_iter().map(|t| t.take()).map(R::from).collect()
}

impl From<crate::ast::TypeDecorators<'_>> for TypeDecorators {
    fn from(value: crate::ast::TypeDecorators) -> Self {
        match value {
            crate::ast::TypeDecorators::TypeArrayExtend(arr) => match arr.size {
                Some(size) => Self::SizedArray(size.take()),
                None => Self::Array,
            },
            crate::ast::TypeDecorators::TypeReferenceExtend(_) => Self::Reference,
            crate::ast::TypeDecorators::TypePointerExtend(_) => Self::Pointer,
        }
    }
}

impl From<crate::ast::TypeDeclare<'_>> for TypeDefinition {
    fn from(value: crate::ast::TypeDeclare) -> Self {
        Self {
            const_: value.const_.is_some(),
            decorators: take_all(value.decorators),
            width: value.width.map(|t| t.width.take()),
            sign: value.sign.map(|t| t.sign),
            type_: value.real_type.take().ident,
        }
    }
}

impl From<crate::ast::Parameter<'_>> for Parameter {
    fn from(value: crate::ast::Parameter<'_>) -> Self {
        Parameter {
            type_: value.type_.take().into(),
            name: value.name.take().ident,
        }
    }
}

impl From<crate::ast::AtomicExpr<'_>> for Expr {
    fn from(value: crate::ast::AtomicExpr<'_>) -> Self {
        match value {
            crate::ast::AtomicExpr::CharLiteral(char) => Self::Char(char.parsed),
            crate::ast::AtomicExpr::StringLiteral(string) => Self::String(string.parsed),
            crate::ast::AtomicExpr::NumberLiteral(number) => match number {
                crate::ast::NumberLiteral::Float { number, .. } => Self::Float(number),
                crate::ast::NumberLiteral::Digit { number, .. } => Self::Digit(number),
            },
            crate::ast::AtomicExpr::Initialization(init) => {
                Self::Initialization(take_all(init.args))
            }
            crate::ast::AtomicExpr::FunctionCall(fn_call) => Self::FuncionCall(FunctionCall {
                name: fn_call.fn_name.take().ident,
                args: take_all(fn_call.args.take().args),
            }),
            crate::ast::AtomicExpr::Variable(var) => Self::Variable(var.ident),
            crate::ast::AtomicExpr::UnaryExpr(unary) => Self::Unary(
                unary.operator.take(),
                Box::new(Expr::from(unary.expr.take())),
            ),
            crate::ast::AtomicExpr::BracketExpr(bracket) => bracket.expr.take().into(),
        }
    }
}

impl From<crate::ast::Expr<'_>> for Expr {
    fn from(value: crate::ast::Expr) -> Self {
        match value {
            crate::ast::Expr::Atomic(atomic) => atomic.into(),
            crate::ast::Expr::Binary(l, o, r) => Self::Binary(
                o.take(),
                Box::new(l.take().into()),
                Box::new(r.take().into()),
            ),
        }
    }
}

impl From<crate::ast::Statement<'_>> for Statement {
    fn from(value: crate::ast::Statement) -> Self {
        match value {
            crate::ast::Statement::VariableDefineStatement(var_def) => {
                let inner = var_def.inner.take();
                Self::VarDef(VariableDefinition {
                    type_: inner.type_.take().into(),
                    name: inner.name.take().ident,
                    init: None,
                })
            }
            crate::ast::Statement::FunctionCallStatement(fn_call) => {
                let fn_call = fn_call.inner.take();
                Self::FnCall(FunctionCall {
                    name: fn_call.fn_name.take().ident,
                    args: take_all(fn_call.args.take().args),
                })
            }
            crate::ast::Statement::VariableInitStatement(init) => {
                let take = init.inner.take();
                let define = take.define.take();
                let init = take.init.take();
                Self::VarDef(VariableDefinition {
                    type_: define.type_.take().into(),
                    name: define.name.take().ident,
                    init: Some(init.value.take().into()),
                })
            }
            crate::ast::Statement::VariableReAssignStatement(ass) => {
                let inner = ass.inner.take();
                Self::VarAss(VariableAssign {
                    name: inner.name.take().ident,
                    assign: inner.assign.take().value.take().into(),
                })
            }
            crate::ast::Statement::If(if_) => {
                let mut branches = vec![];

                let ruo4 = if_.ruo4.take();
                branches.push(IfBranch {
                    conds: take_all(ruo4.conds.take().args),
                    body: take_all(ruo4.block.take().stmts),
                });

                for chain in if_.chains {
                    match chain.take() {
                        crate::ast::ChainIf::AtomicElseIf(else_if) => {
                            let ruo4 = else_if.ruo4.take();
                            branches.push(IfBranch {
                                conds: take_all(ruo4.conds.take().args),
                                body: take_all(ruo4.block.take().stmts),
                            });
                        }
                        crate::ast::ChainIf::AtomicElse(else_) => {
                            return Self::If(If {
                                branches,
                                else_: Some(take_all(else_.block.take().stmts)),
                            })
                        }
                    }
                }
                Self::If(If {
                    branches,
                    else_: None,
                })
            }
            crate::ast::Statement::While(while_) => Self::While(While {
                conds: take_all(while_.conds.take().args),
                body: take_all(while_.block.take().stmts),
            }),
            crate::ast::Statement::Return(return_) => Statement::Return(Return {
                val: return_.val.map(|v| v.take().into()),
            }),
            crate::ast::Statement::FunctionDefine(fn_def) => {
                let fn_ = fn_def.function.take();
                Self::FnDef(FuncionDefinition {
                    type_: fn_.type_.take().into(),
                    name: fn_.name.take().ident,
                    args: take_all(fn_def.params.take().params),
                    body: take_all(fn_def.codes.take().stmts),
                })
            }
            crate::ast::Statement::CodeBlock(stmts) => Self::CodeBlock(take_all(stmts.stmts)),
            crate::ast::Statement::Comment(_) => Self::Empty,
        }
    }
}
