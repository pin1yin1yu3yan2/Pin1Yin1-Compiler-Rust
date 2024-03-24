use crate::keywords::operators::Operators;

pub type Statements = Vec<Statement>;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Statement {
    FnDefine(FnDefine),
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FnDefine {
    #[serde(rename = "type")]
    pub ty: TypeDefine,
    pub name: String,
    pub params: Parameters,
    pub body: Statements,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct VarDefine {
    #[serde(rename = "type")]
    pub ty: TypeDefine,
    pub name: String,
    /// `init` must be an `atomic expr`
    pub init: Option<Expr>,
}

pub type Variable = String;
pub type Variables = Vec<String>;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct VarStore {
    pub name: String,
    pub val: Variable,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FnCall {
    pub name: String,
    pub args: Variables,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Condition {
    // the final value of the condition
    pub val: String,
    pub compute: Statements,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct IfBranch {
    pub cond: Condition,
    pub body: Statements,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct If {
    pub branches: Vec<IfBranch>,
    #[serde(rename = "else")]
    pub else_: Option<Statements>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct While {
    pub cond: Condition,
    pub body: Statements,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Return {
    /// here, this should be [`Option<Variable>`]
    ///
    /// but that's llvm's work! we just bind a literal to a variable,
    /// and return it
    ///
    /// llvm will and should opt this(
    pub val: Option<Variable>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Expr {
    // atomic(both on lex and parse)
    Char(char),
    String(String),
    Integer(usize),
    Float(f64),
    Variable(String),

    // complex
    FuncionCall(FnCall),
    Unary(Operators, Box<Expr>),
    Binary(Operators, Box<Expr>, Box<Expr>),

    // unsupport :)
    Initialization(Vec<Expr>),
}

impl From<String> for Expr {
    fn from(value: String) -> Self {
        Self::Variable(value)
    }
}

impl Expr {
    pub fn binary(op: Operators, l: impl Into<Expr>, r: impl Into<Expr>) -> Self {
        Self::Binary(op, Box::new(l.into()), Box::new(r.into()))
    }

    pub fn unary(op: Operators, l: impl Into<Expr>) -> Self {
        Self::Unary(op, Box::new(l.into()))
    }
}

pub type Parameters = Vec<Parameter>;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Parameter {
    #[serde(rename = "type")]
    pub ty: TypeDefine,
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct TypeDefine {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub decorators: Vec<TypeDecorators>,
    #[serde(rename = "type")]
    pub ty: String,
}

impl TypeDefine {
    pub fn integer() -> Self {
        Self {
            decorators: vec![],
            ty: "i64".into(),
        }
    }
    pub fn float() -> Self {
        Self {
            decorators: vec![],
            ty: "f32".into(),
        }
    }
    pub fn char() -> Self {
        Self {
            decorators: vec![],
            ty: "zi4".into(),
        }
    }
    pub fn string() -> Self {
        Self {
            decorators: vec![TypeDecorators::Array],
            ty: "zi4".into(),
        }
    }
    pub fn bool() -> Self {
        Self {
            decorators: vec![],
            ty: "bu4".into(),
        }
    }

    #[deprecated = "this is not going to be implemented in pin1yin1"]
    pub fn complex() -> Self {
        Self {
            decorators: vec![],
            ty: "xu1".into(),
        }
    }
}

impl std::fmt::Display for TypeDefine {
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

#[cfg(feature = "parser")]
impl<'s> TryFrom<crate::parse::TypeDefine<'s>> for TypeDefine {
    type Error = pin1yin1_parser::ParseError<'s>;

    fn try_from(value: crate::parse::TypeDefine<'s>) -> Result<Self, Self::Error> {
        use pin1yin1_parser::{ErrorKind, WithSelection};
        /*
           int: sign, width
           float: width
        */

        let ty = if &**value.ty == "zheng3" {
            // default to be i64
            let sign = value.sign.map(|pu| pu.sign).unwrap_or(true);
            let sign_char = if sign { 'i' } else { 'u' };
            let width = if let Some(width) = value.width {
                if !width.width.is_power_of_two() || *width.width > 64 {
                    return Err(width.make_error(
                        format!("`zheng3` with width {} is not suppert now", *width.width),
                        ErrorKind::Semantic,
                    ));
                }
                *width.width
            } else {
                64
            };
            value.width.map(|pu| *pu.width).unwrap_or(64);

            format!("{sign_char}{width}")
        } else if &**value.ty == "fu2" {
            // default to be f32
            if let Some(sign) = value.sign {
                return Err(sign.make_error(
                    "`fu2` type cant be decorated with `you3fu2` or `wu2fu2`",
                    ErrorKind::Semantic,
                ));
            }
            let width = if let Some(width) = value.width {
                if *width.width == 32 || *width.width == 64 {
                    *width.width
                } else {
                    return Err(width.make_error(
                        format!("`fu2` with width {} is not supperted now", *width.width),
                        ErrorKind::Semantic,
                    ));
                }
            } else {
                32
            };
            format!("f{width}")
        } else {
            if let Some(sign) = value.sign {
                return Err(sign.make_error(
                    format!(
                        "type `{}` with `you3fu2` or `wu2fu2` is not supperted now",
                        value.ty.ident
                    ),
                    ErrorKind::Semantic,
                ));
            }
            if let Some(width) = value.width {
                return Err(width.make_error(
                    format!(
                        "type `{}` with `you3fu2` or `wu2fu2` is not supperted now",
                        value.ty.ident
                    ),
                    ErrorKind::Semantic,
                ));
            }
            value.ty.take().ident
        };

        let mut decorators = vec![];
        if value.const_.is_some() {
            decorators.push(TypeDecorators::Const);
        }

        for decorator in value.decorators {
            let decorator = match decorator.take() {
                crate::parse::TypeDecorators::TypeArrayExtend(array) => match array.size {
                    Some(size) => TypeDecorators::SizedArray(size.take()),
                    None => TypeDecorators::Array,
                },
                crate::parse::TypeDecorators::TypeReferenceExtend(_) => TypeDecorators::Reference,
                crate::parse::TypeDecorators::TypePointerExtend(_) => TypeDecorators::Pointer,
            };
            decorators.push(decorator);
        }

        Ok(Self { decorators, ty })
    }
}
