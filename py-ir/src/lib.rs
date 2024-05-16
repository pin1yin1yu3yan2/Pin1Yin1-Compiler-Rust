pub mod types;
pub mod value;

use py_lex::{ops::Operators, SharedString};

pub trait IRValue {
    type ComputeType: serde::Serialize
        + for<'a> serde::Deserialize<'a>
        + std::fmt::Debug
        + Clone
        + PartialEq;
    type VarDefineType: serde::Serialize
        + for<'a> serde::Deserialize<'a>
        + std::fmt::Debug
        + Clone
        + PartialEq;
    type FnDefineType: serde::Serialize
        + for<'a> serde::Deserialize<'a>
        + std::fmt::Debug
        + Clone
        + PartialEq;
    type ParameterType: serde::Serialize
        + for<'a> serde::Deserialize<'a>
        + std::fmt::Debug
        + Clone
        + PartialEq;
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum Item<Var: IRValue = crate::value::Value> {
    FnDefine(FnDefine<Var>),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Statements<Var: IRValue> {
    pub stmts: Vec<Statement<Var>>,
    pub returned: bool,
}

impl<Var: IRValue> Statements<Var> {
    pub fn new() -> Self {
        Self {
            stmts: vec![],
            returned: false,
        }
    }

    pub fn push(&mut self, stmt: impl Into<Statement<Var>>) {
        let stmt = stmt.into();
        self.returned = self.returned || stmt.returned();
        self.stmts.push(stmt);
    }
}

impl<Var: IRValue> Default for Statements<Var> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Var: IRValue> From<Vec<Statement<Var>>> for Statements<Var> {
    fn from(stmts: Vec<Statement<Var>>) -> Self {
        Statements {
            returned: stmts.iter().any(|stmt| stmt.returned()),
            stmts,
        }
    }
}

/// just make code generation easier
impl<Var: IRValue> std::ops::Deref for Statements<Var> {
    type Target = Vec<Statement<Var>>;

    fn deref(&self) -> &Self::Target {
        &self.stmts
    }
}

pub trait ControlFlow {
    fn returned(&self) -> bool;
}

impl<Var: IRValue> ControlFlow for Statements<Var> {
    fn returned(&self) -> bool {
        self.returned
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum Statement<Var: IRValue> {
    Compute(Compute<Var>),
    VarDefine(VarDefine<Var>),
    VarStore(VarStore<Var>),
    Block(Statements<Var>),
    If(If<Var>),
    While(While<Var>),
    Return(Return<Var>),
}

impl<Var: IRValue> ControlFlow for Statement<Var> {
    fn returned(&self) -> bool {
        match self {
            Statement::Block(v) => v.returned(),
            Statement::If(v) => v.returned(),
            Statement::While(v) => v.returned(),
            Statement::Return(v) => v.returned(),
            _ => false,
        }
    }
}

mod from_impls {
    use super::*;
    impl<Var: IRValue> From<FnDefine<Var>> for Item<Var> {
        fn from(v: FnDefine<Var>) -> Self {
            Self::FnDefine(v)
        }
    }

    impl<Var: IRValue> From<Compute<Var>> for Statement<Var> {
        fn from(v: Compute<Var>) -> Self {
            Self::Compute(v)
        }
    }

    impl<Var: IRValue> From<VarDefine<Var>> for Statement<Var> {
        fn from(v: VarDefine<Var>) -> Self {
            Self::VarDefine(v)
        }
    }

    impl<Var: IRValue> From<VarStore<Var>> for Statement<Var> {
        fn from(v: VarStore<Var>) -> Self {
            Self::VarStore(v)
        }
    }

    impl<Var: IRValue> From<Statements<Var>> for Statement<Var> {
        fn from(v: Statements<Var>) -> Self {
            Self::Block(v)
        }
    }

    impl<Var: IRValue> From<If<Var>> for Statement<Var> {
        fn from(v: If<Var>) -> Self {
            Self::If(v)
        }
    }

    impl<Var: IRValue> From<While<Var>> for Statement<Var> {
        fn from(v: While<Var>) -> Self {
            Self::While(v)
        }
    }

    impl<Var: IRValue> From<Return<Var>> for Statement<Var> {
        fn from(v: Return<Var>) -> Self {
            Self::Return(v)
        }
    }
}

/// [`OperateExpr::Unary`] and [`OperateExpr::Binary`] are normal operations aroud primitives
///
/// computes around non-primitive types are turned into [FnCall]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum OperateExpr<Var> {
    Unary(Operators, Var),
    Binary(Operators, Var, Var),
}

impl<Var: IRValue> OperateExpr<Var> {
    pub fn binary(op: Operators, l: impl Into<Var>, r: impl Into<Var>) -> Self {
        Self::Binary(op, l.into(), r.into())
    }

    pub fn unary(op: Operators, v: impl Into<Var>) -> Self {
        Self::Unary(op, v.into())
    }
}

/// computing aroud primitive types
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Compute<Var: IRValue> {
    /// computing is around same types
    ///
    /// this is [`OperateExpr::Binary`] or [`OperateExpr::Unary`]'s type
    ///
    /// although [`Variable::Literal`] own its type, [`Variable::FnCall`] and other are not
    #[serde(rename = "type")]
    pub ty: Var::ComputeType,
    pub name: SharedString,
    pub eval: OperateExpr<Var>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Parameter<Pty> {
    #[serde(rename = "type")]
    pub ty: Pty,
    /// using string because its the name of parameter, not a value
    pub name: SharedString,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct FnDefine<Var: IRValue> {
    #[serde(rename = "type")]
    pub ty: Var::FnDefineType,
    pub name: SharedString,
    pub params: Vec<Parameter<Var::ParameterType>>,
    pub body: Statements<Var>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct VarDefine<Var: IRValue> {
    #[serde(rename = "type")]
    pub ty: Var::VarDefineType,
    pub name: SharedString,
    pub init: Option<Var>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct VarStore<Var> {
    pub name: SharedString,
    pub val: Var,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Condition<Var: IRValue> {
    // the final value of the condition
    pub val: Var,
    pub compute: Statements<Var>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct IfBranch<Var: IRValue> {
    pub cond: Condition<Var>,
    pub body: Statements<Var>,
}

impl<Var: IRValue> ControlFlow for IfBranch<Var> {
    fn returned(&self) -> bool {
        self.body.returned()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct If<Var: IRValue> {
    pub branches: Vec<IfBranch<Var>>,
    #[serde(rename = "else")]
    pub else_: Option<Statements<Var>>,
}

impl<Var: IRValue> ControlFlow for If<Var> {
    fn returned(&self) -> bool {
        self.else_.as_ref().is_some_and(|else_| else_.returned())
            && self.branches.iter().all(|branch| branch.returned())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct While<Var: IRValue> {
    pub cond: Condition<Var>,
    pub body: Statements<Var>,
}

impl<Var: IRValue> ControlFlow for While<Var> {
    fn returned(&self) -> bool {
        false
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Return<Var> {
    pub val: Option<Var>,
}

impl<Var: IRValue> ControlFlow for Return<Var> {
    fn returned(&self) -> bool {
        true
    }
}

#[macro_export]
macro_rules! custom_ir_variable {
    ($vis:vis IR<$variable:ty>) => {
        $vis type Item        = $crate::Item        <$variable>;
        $vis type FnDefine    = $crate::FnDefine    <$variable>;
        $vis type Statements  = $crate::Statements  <$variable>;
        $vis type Statement   = $crate::Statement   <$variable>;
        $vis type Compute     = $crate::Compute     <$variable>;
        $vis type VarDefine   = $crate::VarDefine   <$variable>;
        $vis type VarStore    = $crate::VarStore    <$variable>;
        $vis type Condition   = $crate::Condition   <$variable>;
        $vis type If          = $crate::If          <$variable>;
        $vis type IfBranch    = $crate::IfBranch    <$variable>;
        $vis type While       = $crate::While       <$variable>;
        $vis type Return      = $crate::Return      <$variable>;
        $vis type OperateExpr = $crate::OperateExpr <$variable>;
        $vis type Parameter   = $crate::Parameter   <<$variable as $crate::IRValue>::ParameterType>;
    };
}
