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
