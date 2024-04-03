use std::borrow::Cow;

use py_ir::ir::PrimitiveType;

use crate::ops::Operators;

use super::declare::*;

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

#[derive(Debug, Clone)]
pub enum AtomicExpr {
    Char(char),
    String(String),
    Integer(usize),
    Float(f64),
    Variable(String),
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

impl DeclareKind for Type {
    type Type = Type;
}

#[derive(Debug, Clone)]
pub struct NormalVariable {
    pub expr: AtomicExpr,
    pub ty: DeclareIdx,
}

impl NormalVariable {
    pub fn default_declare() -> &'static Type {
        todo!()
    }
}

impl Declare<Type> for NormalVariable {
    fn get_declare_idx(&self) -> DeclareIdx {
        self.ty
    }

    unsafe fn solve<'a>(
        &'a mut self,
        map: &'a mut DeclareMap,
    ) -> Option<&'a <Type as DeclareKind>::Type> {
        match map.solve_one::<Type>(self.get_declare_idx()) {
            Some(..) => self.declare_result(map),
            None => {
                let declare = self.get_declarer(map);
                match self.expr {
                    AtomicExpr::Integer(_) if declare.contain(map, &Type::DEFAULT_INT) => {
                        Some(&Type::DEFAULT_INT)
                    }
                    AtomicExpr::Float(_) if declare.contain(map, &Type::DEFAULT_FLOAT) => {
                        Some(&Type::DEFAULT_FLOAT)
                    }
                    _ => None,
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Variable {
    Normal(NormalVariable),
    FnCall(FnCall),
}

pub type Variables = Vec<Variable>;

/// be fferent of [py_ir::ir::OperateExpr], this is **not** around primitive types
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
pub struct ComplexType {
    pub decorators: Option<Vec<TypeDecorators>>,
    pub ty: String,
}

impl ComplexType {
    pub fn no_decorators(ty: String) -> Self {
        Self {
            decorators: None,
            ty,
        }
    }

    pub fn string() -> Self {
        Self {
            decorators: Some(vec![TypeDecorators::Array]),
            ty: "u8".to_string(),
        }
    }
}

impl std::fmt::Display for ComplexType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(decorators) = &self.decorators {
            for dec in decorators {
                match dec {
                    TypeDecorators::Const => write!(f, "const "),
                    TypeDecorators::Array => write!(f, "[] "),
                    TypeDecorators::Reference => write!(f, "& "),
                    TypeDecorators::Pointer => write!(f, "* "),
                    TypeDecorators::SizedArray(s) => write!(f, "[{s}] "),
                }?;
            }
        }

        write!(f, "{}", self.ty)
    }
}

#[derive(Debug, Clone)]
pub enum TypeDefine {
    Primitive(PrimitiveType),
    Complex(ComplexType),
}

impl TypeDefine {
    /// Returns `true` if the type define is [`Primitive`].
    ///
    /// [`Primitive`]: TypeDefine::Primitive
    #[must_use]
    pub fn is_primitive(&self) -> bool {
        matches!(self, Self::Primitive(..))
    }

    /// Returns `true` if the type define is [`Complex`].
    ///
    /// [`Complex`]: TypeDefine::Complex
    #[must_use]
    pub fn is_complex(&self) -> bool {
        matches!(self, Self::Complex(..))
    }
}

impl std::fmt::Display for TypeDefine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeDefine::Primitive(ty) => write!(f, "{}", ty),
            TypeDefine::Complex(ty) => write!(f, "{}", ty),
        }
    }
}

impl From<ComplexType> for TypeDefine {
    fn from(v: ComplexType) -> Self {
        Self::Complex(v)
    }
}

impl From<PrimitiveType> for TypeDefine {
    fn from(v: PrimitiveType) -> Self {
        Self::Primitive(v)
    }
}

impl TryFrom<TypeDefine> for PrimitiveType {
    type Error = TypeDefine;

    fn try_from(value: TypeDefine) -> Result<Self, Self::Error> {
        match value {
            TypeDefine::Primitive(p) => Ok(p),
            TypeDefine::Complex(_) => Err(value),
        }
    }
}

impl TryFrom<TypeDefine> for ComplexType {
    type Error = TypeDefine;

    fn try_from(value: TypeDefine) -> Result<Self, Self::Error> {
        match value {
            TypeDefine::Primitive(_) => Err(value),
            TypeDefine::Complex(c) => Ok(c),
        }
    }
}

impl PartialEq<PrimitiveType> for TypeDefine {
    fn eq(&self, other: &PrimitiveType) -> bool {
        match self {
            TypeDefine::Primitive(s) => s == other,
            TypeDefine::Complex(_) => false,
        }
    }
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

pub struct FnOverloadSelect {
    mangled: String,
}

impl DeclareKind for FnOverloadSelect {
    type Type = String;
}

#[derive(Debug, Clone)]
pub struct FnCall {
    /// FnOverloadSelect
    pub fn_name: DeclareIdx,
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
