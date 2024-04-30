use crate::ops::Operators;

#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone, PartialEq)]
pub struct Statements {
    pub stmts: Vec<Statement>,
    pub returned: bool,
}

/// just make code generation easier
impl std::ops::Deref for Statements {
    type Target = Vec<Statement>;

    fn deref(&self) -> &Self::Target {
        &self.stmts
    }
}

pub trait ControlFlow {
    fn returned(&self) -> bool;
}

impl ControlFlow for Statements {
    fn returned(&self) -> bool {
        self.returned
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum Item {
    FnDefine(FnDefine),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum Statement {
    Compute(Compute),
    VarDefine(VarDefine),
    VarStore(VarStore),
    Block(Statements),
    If(If),
    While(While),
    Return(Return),
}

impl ControlFlow for Statement {
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
    impl From<FnDefine> for Item {
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
/// type of literals are needed to be declared in operators, because `1` can mean `i8`, `i32`, etc.
///
/// [`Variable::Variable`] and [`Variable::FnCall`] are folded expression,for example,
/// [`OperateExpr::Binary`] and [`OperateExpr::Unary`] will be transformed into a [`VarDefine`],
/// and its result(a variable) will be used as [`Variable::Variable`]
///
/// using this way to avoid expressions' tree, and make llvm-ir generation much easier
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum Variable {
    Variable(String),
    FnCall(FnCall),
    // #[deprecated = "unsupported now"]
    // Initialization(Vec<Expr>),
    Literal(Literal, PrimitiveType),
}

/// [`Literal::Char`], [`Literal::Integer`] and [`Literal::Float`]
/// mean literals
///
/// althogn [`String`] is also [`Literal`], it will be replaced with [`VarDefine`] statement
/// so that the type of [`Literal`] can be represented by [`PrimitiveType`]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum Literal {
    Char(char),
    Integer(usize),
    Float(f64),
}

pub type Variables = Vec<Variable>;

/// [`OperateExpr::Unary`] and [`OperateExpr::Binary`] are normal operations aroud primitives
///
/// computes around non-primitive types are turned into [FnCall]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum OperateExpr {
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
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

    pub fn width(&self) -> usize {
        match self {
            PrimitiveType::Bool => 1,
            PrimitiveType::I8 | PrimitiveType::U8 => 8,
            PrimitiveType::I16 | PrimitiveType::U16 => 16,
            PrimitiveType::I32 | PrimitiveType::U32 => 32,
            PrimitiveType::I64 | PrimitiveType::U64 => 64,
            PrimitiveType::I128 | PrimitiveType::U128 => 128,
            // !
            PrimitiveType::Usize | PrimitiveType::Isize => 64,
            PrimitiveType::F32 => 32,
            PrimitiveType::F64 => 64,
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
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Compute {
    /// computing is around same types
    ///
    /// this is [`OperateExpr::Binary`] or [`OperateExpr::Unary`]'s type
    ///
    /// although [`Variable::Literal`] own its type, [`Variable::FnCall`] and other are not
    #[serde(rename = "type")]
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct ComplexType {
    /// use option to avoid memory allocation sometimes
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub decorators: Vec<TypeDecorators>,
    #[serde(rename = "type")]
    pub ty: String,
}

impl ComplexType {
    pub fn no_decorators(ty: String) -> Self {
        Self {
            decorators: Vec::new(),
            ty,
        }
    }

    pub fn string() -> Self {
        Self {
            decorators: vec![TypeDecorators::Array],
            ty: "u8".to_string(),
        }
    }
}

impl std::fmt::Display for ComplexType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for dec in &self.decorators {
            match dec {
                TypeDecorators::Const => write!(f, "const "),
                TypeDecorators::Array => write!(f, "[] "),
                TypeDecorators::Reference => write!(f, "& "),
                TypeDecorators::Pointer => write!(f, "* "),
                TypeDecorators::SizedArray(s) => write!(f, "[{s}] "),
            }?;
        }

        write!(f, "{}", self.ty)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
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

    pub fn as_primitive(&self) -> Option<&PrimitiveType> {
        if let Self::Primitive(v) = self {
            Some(v)
        } else {
            None
        }
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Parameter {
    #[serde(rename = "type")]
    pub ty: TypeDefine,
    /// using string because its the name of parameter, not a value
    pub name: String,
}

pub type Parameters = Vec<Parameter>;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct FnDefine {
    #[serde(rename = "type")]
    pub ty: TypeDefine,
    pub name: String,
    pub params: Parameters,
    pub body: Statements,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct VarDefine {
    #[serde(rename = "type")]
    pub ty: TypeDefine,
    pub name: String,
    pub init: Option<Variable>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct VarStore {
    pub name: String,
    pub val: Variable,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct FnCall {
    #[serde(rename = "fn")]
    pub fn_name: String,
    pub args: Variables,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Condition {
    // the final value of the condition
    pub val: Variable,
    pub compute: Statements,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct IfBranch {
    pub cond: Condition,
    pub body: Statements,
}

impl ControlFlow for IfBranch {
    fn returned(&self) -> bool {
        self.body.returned()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct If {
    pub branches: Vec<IfBranch>,
    #[serde(rename = "else")]
    pub else_: Option<Statements>,
}

impl ControlFlow for If {
    fn returned(&self) -> bool {
        self.else_.as_ref().is_some_and(|else_| else_.returned())
            && self.branches.iter().all(|branch| branch.returned())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct While {
    pub cond: Condition,
    pub body: Statements,
}

impl ControlFlow for While {
    fn returned(&self) -> bool {
        false
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Return {
    pub val: Option<Variable>,
}

impl ControlFlow for Return {
    fn returned(&self) -> bool {
        true
    }
}

mod serde_ {
    use super::*;
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = TypeDecorators;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("Const, Unigned, Array, Reference, Pointer...")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match v {
                "Const" => Ok(TypeDecorators::Const),
                "Array" => Ok(TypeDecorators::Array),
                "Reference" => Ok(TypeDecorators::Reference),
                "Pointer" => Ok(TypeDecorators::Pointer),
                a => a
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
                TypeDecorators::Const => serializer.serialize_str("Const"),
                TypeDecorators::Array => serializer.serialize_str("Array"),
                TypeDecorators::Reference => serializer.serialize_str("Reference"),
                TypeDecorators::Pointer => serializer.serialize_str("Pointer"),
                TypeDecorators::SizedArray(v) => serializer.serialize_str(&format!("Array {v}")),
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
