use std::fmt::Write;

use py_lex::{ops::Operators, SharedString};

use crate::types::{PrimitiveType, TypeDefine};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Value {
    Variable(SharedString),
    Literal(Literal, PrimitiveType),
}

/// [`Operate::Unary`] and [`Operate::Binary`] are normal operations aroud primitives
///
/// computes around non-primitive types are turned into [FnCall]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Operate {
    Unary(Operators, Value),
    Binary(Operators, Value, Value),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum AssignValue {
    Value(Value),
    FnCall(FnCall<Value>),
    Operate(Operate, PrimitiveType),
}

impl From<Value> for AssignValue {
    fn from(v: Value) -> Self {
        Self::Value(v)
    }
}

impl From<FnCall<Value>> for AssignValue {
    fn from(v: FnCall<Value>) -> Self {
        Self::FnCall(v)
    }
}

impl From<(Operate, PrimitiveType)> for AssignValue {
    fn from(value: (Operate, PrimitiveType)) -> Self {
        Self::Operate(value.0, value.1)
    }
}

impl super::IRValue for Value {
    type AssignValue = AssignValue;
    type VarDefineType = TypeDefine;
    type FnDefineType = TypeDefine;
    type ParameterType = TypeDefine;
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FnCall<Var> {
    #[serde(rename = "fn")]
    pub fn_name: SharedString,
    pub args: Vec<Var>,
}

/// [`Literal::Char`], [`Literal::Integer`] and [`Literal::Float`]
/// mean literals
///
/// althogn [`String`] is also [`Literal`], it will be replaced with [`VarDefine`] statement
/// so that the type of [`Literal`] can be represented by [`PrimitiveType`]
///
/// [`VarDefine`]: super
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Literal {
    Char(char),
    Integer(usize),
    Float(f64),
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Char(ch) => f.write_char(*ch),
            Literal::Integer(nu) => f.write_fmt(format_args!("{nu}")),
            Literal::Float(fl) => f.write_fmt(format_args!("{fl}")),
        }
    }
}
