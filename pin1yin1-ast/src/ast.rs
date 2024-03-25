use crate::keywords::operators::Operators;

pub type Statements = Vec<Statement>;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct FnDefine {
    #[serde(rename = "type")]
    pub ty: TypeDefine,
    pub name: String,
    pub params: Parameters,
    pub body: Statements,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Compute {
    // we can know this in code generation
    // #[serde(rename = "type")]
    // pub ty: TypeDefine,
    pub name: String,
    pub eval: OperateExpr,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct VarDefine {
    #[serde(rename = "type")]
    pub ty: TypeDefine,
    pub name: String,
    /// `init` must be an `atomic expr`
    pub init: Option<Variable>,
}

/// this kind of expr is the most general expression
///
/// [`AtomicExpr::Char`], [`AtomicExpr::String`], [`AtomicExpr::Integer`] and [`AtomicExpr::Float`]
/// mean literals
///
/// [`AtomicExpr::Variable`] and [`AtomicExpr::FnCall`] are folded expression,for example,
/// [`OperateExpr::Binary`] and [`OperateExpr::Unary`] will be transformed into a [`VarDefine`],
/// and its result(a variable) will be treated as [`AtomicExpr::Variable`]
///
/// using this way to avoid expressions' tree, and make llvm-ir generation much easier
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
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

impl AtomicExpr {
    pub fn with_ty(self, ty: TypeDefine) -> TypedExpr {
        TypedExpr { ty, val: self }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct TypedExpr {
    #[serde(rename = "type")]
    pub ty: TypeDefine,
    pub val: AtomicExpr,
}

impl TypedExpr {
    pub fn new(ty: TypeDefine, expr: impl Into<AtomicExpr>) -> Self {
        Self {
            ty,
            val: expr.into(),
        }
    }
}

pub type Variable = AtomicExpr;
pub type Variables = Vec<Variable>;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct VarStore {
    pub name: String,
    pub val: Variable,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct FnCall {
    pub name: String,
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct If {
    pub branches: Vec<IfBranch>,
    #[serde(rename = "else")]
    pub else_: Option<Statements>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct While {
    pub cond: Condition,
    pub body: Statements,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Return {
    /// here, this should be [`Option<Variable>`]
    ///
    /// but that's llvm's work! we just bind a literal to a variable,
    /// and return it
    ///
    /// llvm will and should opt this(
    pub val: Option<Variable>,
}

/// [`OperateExpr::Unary`] and [`OperateExpr::Binary`] are normal operations aroud
/// primitives
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Parameter {
    #[serde(rename = "type")]
    pub ty: TypeDefine,
    /// using string because its the name of parameter, not a value
    pub name: String,
}

pub type Parameters = Vec<Parameter>;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct TypeDefine {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decorators: Option<Vec<TypeDecorators>>,
    #[serde(rename = "type")]
    pub ty: String,
}

impl TypeDefine {
    pub fn no_decorators(ty: impl Into<String>) -> Self {
        Self {
            decorators: None,
            ty: ty.into(),
        }
    }

    pub fn integer() -> Self {
        Self::no_decorators("i64")
    }
    pub fn float() -> Self {
        Self::no_decorators("f32")
    }
    pub fn char() -> Self {
        Self::no_decorators("zi4")
    }
    pub fn string() -> Self {
        Self {
            decorators: vec![TypeDecorators::Array].into(),
            ty: "zi4".into(),
        }
    }
    pub fn bool() -> Self {
        Self::no_decorators("bu4")
    }

    #[deprecated = "this is not going to be implemented in pin1yin1"]
    pub fn complex() -> Self {
        Self::no_decorators("zu1")
    }
}

impl std::fmt::Display for TypeDefine {
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
pub enum TypeDecorators {
    // #[deprecated = "unclear semantics"]
    Const,
    // TODO: remove this varient
    // hmm, `kuan1` has been removed, lol
    Array,
    Reference,
    Pointer,
    SizedArray(usize),
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
