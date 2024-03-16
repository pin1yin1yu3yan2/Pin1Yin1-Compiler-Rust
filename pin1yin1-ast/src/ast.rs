use crate::keywords::operators::Operators;

pub type Statements = Vec<Statement>;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[repr(C)]
pub enum Statement {
    FnDefine(FnDefine),
    VarDefine(VarDefine),
    VarStore(VarStore),
    FnCall(FnCall),
    Block(Statements),
    If(If),
    While(While),
    Return(Return),
    // comment
    Empty,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[repr(C)]
pub struct FnDefine {
    #[serde(rename = "type")]
    pub type_: TypeDefine,
    pub name: String,
    pub args: Parameters,
    pub body: Statements,
}

// TODO: devied this into two ast...
// may ast should be abstract enough to means multiple instructions?
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[repr(C)]
pub struct VarDefine {
    #[serde(rename = "type")]
    pub type_: TypeDefine,
    pub name: String,
    /// TODO: `init` must be an `atomic expr`
    pub init: Option<Expr>,
}

impl VarDefine {
    pub fn new_alloc(type_: TypeDefine, init: impl Into<Option<Expr>>) -> Self {
        use std::sync::atomic::AtomicUsize;
        static ALLOC_NAME: AtomicUsize = AtomicUsize::new(0);
        Self {
            type_,
            init: init.into(),
            name: format!(
                ".{}",
                ALLOC_NAME.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ),
        }
    }
}

pub type Variable = String;
pub type Variables = Vec<String>;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[repr(C)]
pub struct VarStore {
    pub name: String,
    pub value: Variable,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[repr(C)]
pub struct FnCall {
    pub name: String,
    pub args: Variables,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[repr(C)]
pub struct IfBranch {
    pub conds: Vec<Expr>,
    pub body: Statements,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[repr(C)]
pub struct If {
    pub branches: Vec<IfBranch>,
    #[serde(rename = "else")]
    pub else_: Option<Statements>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[repr(C)]
pub struct While {
    pub conds: Vec<Expr>,
    pub body: Statements,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[repr(C)]
pub struct Return {
    pub val: Option<Expr>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[repr(C)]
pub enum Expr {
    Char(char),
    String(String),
    Integer(usize),
    Float(f64),
    Variable(String),
    FuncionCall(FnCall),
    Unary(Operators, Box<Expr>),
    Binary(Operators, Box<Expr>, Box<Expr>),
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
#[repr(C)]
pub struct Parameter {
    #[serde(rename = "type")]
    pub type_: TypeDefine,
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
#[repr(C)]
pub struct TypeDefine {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub decorators: Vec<TypeDecorators>,
    #[serde(rename = "type")]
    pub type_: String,
}

impl TypeDefine {
    pub fn integer() -> Self {
        Self {
            decorators: vec![],
            type_: "zheng3".into(),
        }
    }
    pub fn float() -> Self {
        Self {
            decorators: vec![],
            type_: "fu2".into(),
        }
    }
    pub fn char() -> Self {
        Self {
            decorators: vec![],
            type_: "zi4".into(),
        }
    }
    pub fn string() -> Self {
        Self {
            decorators: vec![TypeDecorators::Array],
            type_: "zi4".into(),
        }
    }
    pub fn bool() -> Self {
        Self {
            decorators: vec![],
            type_: "bu4".into(),
        }
    }

    /// this is not going to be implemented in pin1yin1
    #[deprecated]
    pub fn complex() -> Self {
        Self {
            decorators: vec![],
            type_: "xu1".into(),
        }
    }
}

impl std::fmt::Display for TypeDefine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for dec in &self.decorators {
            match dec {
                TypeDecorators::Const => write!(f, "const "),
                TypeDecorators::Unsigned => write!(f, "unsigned "),
                TypeDecorators::Array => write!(f, "[] "),
                TypeDecorators::Width(w) => write!(f, "[{w}] "),
                TypeDecorators::Reference => write!(f, "& "),
                TypeDecorators::Pointer => write!(f, "* "),
                TypeDecorators::SizedArray(s) => write!(f, "[{s}] "),
            }?;
        }
        write!(f, "{}", self.type_)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDecorators {
    Const,
    Unsigned,
    // TODO: remove this varient
    Array,
    Width(usize),
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
                "Unsigned" => Ok(TypeDecorators::Unsigned),
                "Array" => Ok(TypeDecorators::Array),
                "Reference" => Ok(TypeDecorators::Reference),
                "Pointer" => Ok(TypeDecorators::Pointer),
                a if a.starts_with("Array") => a
                    .split_ascii_whitespace()
                    .nth(1)
                    .ok_or_else(|| E::custom("invalid decorators"))?
                    .parse::<usize>()
                    .map(TypeDecorators::SizedArray)
                    .map_err(E::custom),
                w if w.starts_with("Width") => w
                    .split_ascii_whitespace()
                    .nth(1)
                    .ok_or_else(|| E::custom("invalid decorators"))?
                    .parse::<usize>()
                    .map(TypeDecorators::SizedArray)
                    .map_err(E::custom),

                _ => Err(E::custom("invalid decorators")),
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
                TypeDecorators::Unsigned => serializer.serialize_str("Unsigned"),
                TypeDecorators::Width(v) => serializer.serialize_str(&format!("Width {v}")),
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

impl From<crate::parse::TypeDeclare<'_>> for TypeDefine {
    fn from(value: crate::parse::TypeDeclare) -> Self {
        let mut decorators = vec![];
        decorators.extend(value.const_.map(|_| TypeDecorators::Const));
        decorators.extend(value.decorators.into_iter().map(|d| match d.take() {
            crate::parse::TypeDecorators::TypeArrayExtend(array) => match array.size {
                Some(size) => TypeDecorators::SizedArray(size.take()),
                None => TypeDecorators::Array,
            },
            crate::parse::TypeDecorators::TypeReferenceExtend(_) => TypeDecorators::Reference,
            crate::parse::TypeDecorators::TypePointerExtend(_) => TypeDecorators::Pointer,
        }));
        if value.sign.is_some_and(|sign| !sign.sign) {
            decorators.push(TypeDecorators::Unsigned);
        }
        Self {
            decorators,
            type_: value.real_type.take().ident,
        }
    }
}
