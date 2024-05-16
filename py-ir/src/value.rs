use py_lex::SharedString;

use crate::types::{PrimitiveType, TypeDefine};

/// this kind of expr is the most general expression
///
/// type of literals are needed to be declared in operators, because `1` can mean `i8`, `i32`, etc.
///
/// [`Variable::Variable`] and [`Variable::FnCall`] are folded expression, for example,
/// [`OperateExpr::Binary`] and [`OperateExpr::Unary`] will be transformed into a [`VarDefine`],
/// and its result(a variable) will be used as [`Variable::Variable`]
///
/// using this way to avoid expressions' tree, and make llvm-ir generation much easier
///
/// [`OperateExpr::Binary`]: super
/// [`OperateExpr::unary`]: super
/// [`VarDefine`]: super
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum Value {
    Variable(SharedString),
    FnCall(FnCall<Self>),
    Literal(Literal, PrimitiveType),
}

impl super::IRValue for Value {
    type ComputeType = PrimitiveType;
    type VarDefineType = TypeDefine;
    type FnDefineType = TypeDefine;
    type ParameterType = TypeDefine;
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
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
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum Literal {
    Char(char),
    Integer(usize),
    Float(f64),
}
