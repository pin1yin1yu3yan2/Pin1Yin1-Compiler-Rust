use py_lex::{ops::Operators, SharedString};

pub trait IRVariable {
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

pub type Items<Var> = Vec<Item<Var>>;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum Item<Var: IRVariable> {
    FnDefine(FnDefine<Var>),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Statements<Var: IRVariable = Variable> {
    pub stmts: Vec<Statement<Var>>,
    pub returned: bool,
}

impl<Var: IRVariable> Statements<Var> {
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

impl<Var: IRVariable> Default for Statements<Var> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Var: IRVariable> From<Vec<Statement<Var>>> for Statements<Var> {
    fn from(stmts: Vec<Statement<Var>>) -> Self {
        Statements {
            returned: stmts.iter().any(|stmt| stmt.returned()),
            stmts,
        }
    }
}

/// just make code generation easier
impl<Var: IRVariable> std::ops::Deref for Statements<Var> {
    type Target = Vec<Statement<Var>>;

    fn deref(&self) -> &Self::Target {
        &self.stmts
    }
}

pub trait ControlFlow {
    fn returned(&self) -> bool;
}

impl<Var: IRVariable> ControlFlow for Statements<Var> {
    fn returned(&self) -> bool {
        self.returned
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum Statement<Var: IRVariable> {
    Compute(Compute<Var>),
    VarDefine(VarDefine<Var>),
    VarStore(VarStore<Var>),
    Block(Statements<Var>),
    If(If<Var>),
    While(While<Var>),
    Return(Return<Var>),
}

impl<Var: IRVariable> ControlFlow for Statement<Var> {
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
    impl<Var: IRVariable> From<FnDefine<Var>> for Item<Var> {
        fn from(v: FnDefine<Var>) -> Self {
            Self::FnDefine(v)
        }
    }

    impl<Var: IRVariable> From<Compute<Var>> for Statement<Var> {
        fn from(v: Compute<Var>) -> Self {
            Self::Compute(v)
        }
    }

    impl<Var: IRVariable> From<VarDefine<Var>> for Statement<Var> {
        fn from(v: VarDefine<Var>) -> Self {
            Self::VarDefine(v)
        }
    }

    impl<Var: IRVariable> From<VarStore<Var>> for Statement<Var> {
        fn from(v: VarStore<Var>) -> Self {
            Self::VarStore(v)
        }
    }

    impl<Var: IRVariable> From<Statements<Var>> for Statement<Var> {
        fn from(v: Statements<Var>) -> Self {
            Self::Block(v)
        }
    }

    impl<Var: IRVariable> From<If<Var>> for Statement<Var> {
        fn from(v: If<Var>) -> Self {
            Self::If(v)
        }
    }

    impl<Var: IRVariable> From<While<Var>> for Statement<Var> {
        fn from(v: While<Var>) -> Self {
            Self::While(v)
        }
    }

    impl<Var: IRVariable> From<Return<Var>> for Statement<Var> {
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

impl<Var: IRVariable> OperateExpr<Var> {
    pub fn binary(op: Operators, l: impl Into<Var>, r: impl Into<Var>) -> Self {
        Self::Binary(op, l.into(), r.into())
    }

    pub fn unary(op: Operators, v: impl Into<Var>) -> Self {
        Self::Unary(op, v.into())
    }
}

/// computing aroud primitive types
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Compute<Var: IRVariable> {
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
pub struct FnDefine<Var: IRVariable> {
    #[serde(rename = "type")]
    pub ty: Var::FnDefineType,
    pub name: SharedString,
    pub params: Vec<Parameter<Var::ParameterType>>,
    pub body: Statements<Var>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct VarDefine<Var: IRVariable> {
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
pub struct Condition<Var: IRVariable> {
    // the final value of the condition
    pub val: Var,
    pub compute: Statements<Var>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct IfBranch<Var: IRVariable> {
    pub cond: Condition<Var>,
    pub body: Statements<Var>,
}

impl<Var: IRVariable> ControlFlow for IfBranch<Var> {
    fn returned(&self) -> bool {
        self.body.returned()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct If<Var: IRVariable> {
    pub branches: Vec<IfBranch<Var>>,
    #[serde(rename = "else")]
    pub else_: Option<Statements<Var>>,
}

impl<Var: IRVariable> ControlFlow for If<Var> {
    fn returned(&self) -> bool {
        self.else_.as_ref().is_some_and(|else_| else_.returned())
            && self.branches.iter().all(|branch| branch.returned())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct While<Var: IRVariable> {
    pub cond: Condition<Var>,
    pub body: Statements<Var>,
}

impl<Var: IRVariable> ControlFlow for While<Var> {
    fn returned(&self) -> bool {
        false
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Return<Var> {
    pub val: Option<Var>,
}

impl<Var: IRVariable> ControlFlow for Return<Var> {
    fn returned(&self) -> bool {
        true
    }
}

mod serde_type_decorators {
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

pub mod ir_variable {
    use py_lex::SharedString;

    use super::{PrimitiveType, TypeDefine};

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
    pub enum Variable {
        Variable(SharedString),
        FnCall(FnCall<Self>),
        Literal(Literal, PrimitiveType),
    }

    impl super::IRVariable for Variable {
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
}

pub use ir_variable::*;

pub mod ir_types {
    use std::collections::HashMap;

    use super::SharedString;

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
        pub ty: SharedString,
    }

    impl ComplexType {
        pub fn no_decorators(ty: SharedString) -> Self {
            Self {
                decorators: Vec::new(),
                ty,
            }
        }

        pub fn string() -> Self {
            Self {
                decorators: vec![TypeDecorators::Array],
                ty: "u8".into(),
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
    pub enum Type {
        Template(Template),
        Primitive(PrimitiveType),
        Custom(SharedString),
    }

    impl From<Template> for Type {
        fn from(v: Template) -> Self {
            Self::Template(v)
        }
    }

    impl From<PrimitiveType> for Type {
        fn from(v: PrimitiveType) -> Self {
            Self::Primitive(v)
        }
    }

    impl From<SharedString> for Type {
        fn from(v: SharedString) -> Self {
            Self::Custom(v)
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
    pub struct Template {
        pub name: SharedString,
        pub generics: HashMap<SharedString, Type>,
    }

    impl Template {
        pub fn new<I>(name: I, generics: HashMap<SharedString, Type>) -> Self
        where
            I: Into<SharedString>,
        {
            Self {
                name: name.into(),
                generics,
            }
        }

        pub fn reference(to: Type) -> Self {
            Self::new("&", std::iter::once(("T".into(), to)).collect())
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
}

pub use ir_types::*;

#[macro_export]
macro_rules! custom_ir_variable {
    ($vis:vis IR<$variable:ty>) => {
        $vis type Items       = $crate::Items       <$variable>;
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
        $vis type Parameter   = $crate::Parameter   <<$variable as $crate::IRVariable>::ParameterType>;
    };
}
