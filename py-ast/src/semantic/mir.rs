use crate::ops::Operators;

pub type Statements = Vec<Statement>;

#[derive(Debug, Clone, PartialEq)]
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

/// this kind of expr is the most general expression
///
/// [`AtomicExpr::Char`], [`AtomicExpr::String`], [`AtomicExpr::Integer`] and [`AtomicExpr::Float`]
/// mean literals
///
/// type of literals are needed to be declared in operators, because `1` can mean `i8`, `i32`, etc.
///
/// [`AtomicExpr::Variable`] and [`AtomicExpr::FnCall`] are folded expression,for example,
/// [`OperateExpr::Binary`] and [`OperateExpr::Unary`] will be transformed into a [`VarDefine`],
/// and its result(a variable) will be used as [`AtomicExpr::Variable`]
///
/// using this way to avoid expressions' tree, and make llvm-ir generation much easier
#[derive(Debug, Clone, PartialEq)]
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

/// comes from [`Compute`] or just a
pub type Variable = AtomicExpr;
pub type Variables = Vec<Variable>;

/// [`OperateExpr::Unary`] and [`OperateExpr::Binary`] are normal operations aroud primitives
///
/// computes around non-primitive types are turned into [FnCall]
#[derive(Debug, Clone, PartialEq)]
pub enum OperateExpr {
    Unary(Operators, AtomicExpr),
    Binary(Operators, AtomicExpr, AtomicExpr),
}

impl OperateExpr {
    pub fn binary(op: Operators, l: impl Into<AtomicExpr>, r: impl Into<AtomicExpr>) -> Self {
        Self::Binary(op, l.into(), r.into())
    }

    pub fn unary(op: Operators, v: impl Into<AtomicExpr>) -> Self {
        Self::Unary(op, v.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimitiveType {
    Bool, // boolean
    I8,
    U8,
    I16,
    U16,
    I32,
    U32, // char
    I64,
    U64,
    I128,
    U128,
    Usize,
    Isize,
    F32,
    F64,
}

impl PrimitiveType {
    pub fn char() -> Self {
        Self::U32
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Self::F32 | Self::F64)
    }

    pub fn is_integer(&self) -> bool {
        !self.is_float() && self != &Self::Bool
    }

    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::I128 | Self::Isize
        )
    }

    pub fn is_unsigned(&self) -> bool {
        matches!(
            self,
            Self::U8 | Self::U16 | Self::U32 | Self::U64 | Self::U128 | Self::Usize
        )
    }
}

impl std::str::FromStr for PrimitiveType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "i1" => Ok(Self::Bool),
            "i8" => Ok(Self::I8),
            "u8" => Ok(Self::U8),
            "i16" => Ok(Self::I16),
            "u16" => Ok(Self::U16),
            "i32" => Ok(Self::I32),
            "u32" => Ok(Self::U32),
            "i64" => Ok(Self::I64),
            "u64" => Ok(Self::U64),
            "i128" => Ok(Self::I128),
            "u128" => Ok(Self::U128),
            "f32" => Ok(Self::F32),
            "f64" => Ok(Self::F64),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // hmmm...
        write!(f, "{}", format!("{self:?}").to_ascii_lowercase())
    }
}

/// computing aroud primitive types
#[derive(Debug, Clone, PartialEq)]
pub struct Compute {
    /// computing is around same types
    ///
    /// this is [`OperateExpr::Binary`] or [`OperateExpr::Unary`]'s type
    pub ty: PrimitiveType,
    pub name: String,
    pub eval: OperateExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDecorators {
    // #[deprecated = "unclear semantics"]
    Const,
    // TODO: remove this varient
    Array,
    Reference,
    Pointer,
    SizedArray(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComplexType {
    /// use option to avoid memory allocation sometimes
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub ty: TypeDefine,
    /// using string because its the name of parameter, not a value
    pub name: String,
}

pub type Parameters = Vec<Parameter>;

#[derive(Debug, Clone, PartialEq)]
pub struct FnDefine {
    pub ty: TypeDefine,
    pub name: String,
    pub params: Parameters,
    pub body: Statements,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarDefine {
    pub ty: TypeDefine,
    pub name: String,
    pub init: Option<Variable>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarStore {
    pub name: String,
    pub val: Variable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnCall {
    pub fn_name: String,
    pub args: Variables,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    // the final value of the condition
    pub val: Variable,
    pub compute: Statements,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfBranch {
    pub cond: Condition,
    pub body: Statements,
}

#[derive(Debug, Clone, PartialEq)]
pub struct If {
    pub branches: Vec<IfBranch>,

    pub else_: Option<Statements>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct While {
    pub cond: Condition,
    pub body: Statements,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Return {
    pub val: Option<Variable>,
}
