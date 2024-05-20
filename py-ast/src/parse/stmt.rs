use crate::complex_pu;

use super::*;

use py_lex::syntax::Symbol;

#[derive(Debug, Clone)]
pub struct VarAssign {
    pub val: Expr,
}

impl ParseUnit<Token> for VarAssign {
    type Target = VarAssign;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.r#match(Symbol::Assign)?;
        let val = p.parse::<Expr>()?;
        Ok(Self { val })
    }
}

#[derive(Debug, Clone)]
pub struct VarDefine {
    pub ty: PU<types::TypeDefine>,
    pub name: Ident,
    pub init: Option<VarAssign>,
}

impl ParseUnit<Token> for VarDefine {
    type Target = VarDefine;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        let ty = p.parse::<PU<types::TypeDefine>>()?;
        let name = p.parse::<Ident>()?;
        let init = p.parse::<VarAssign>().apply(mapper::Try)?;
        Ok(Self { ty, name, init })
    }
}

#[derive(Debug, Clone)]
pub struct VarStore {
    pub name: Ident,
    pub assign: PU<VarAssign>,
}

impl ParseUnit<Token> for VarStore {
    type Target = VarStore;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        let name = p.parse::<Ident>()?;
        let assign = p.parse::<PU<VarAssign>>()?;
        Ok(VarStore { name, assign })
    }
}

#[derive(Debug, Clone)]
pub struct Parameter {
    /// so to make semantic cheaking easier
    inner: VarDefine,
}

impl std::ops::Deref for Parameter {
    type Target = VarDefine;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ParseUnit<Token> for Parameter {
    type Target = Parameter;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        let ty = p.parse::<PU<types::TypeDefine>>()?;
        let name = p.parse::<Ident>()?;
        let inner = VarDefine {
            ty,
            name,
            init: None,
        };
        Ok(Parameter { inner })
    }
}

#[derive(Debug, Clone)]
pub struct Parameters {
    pub params: Vec<PU<Parameter>>,
}

impl std::ops::Deref for Parameters {
    type Target = Vec<PU<Parameter>>;

    fn deref(&self) -> &Self::Target {
        &self.params
    }
}

impl ParseUnit<Token> for Parameters {
    type Target = Parameters;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.r#match(Symbol::Parameter)?;
        let Some(arg) = p.parse::<PU<Parameter>>().apply(mapper::Try)? else {
            p.r#match(Symbol::EndOfBlock).apply(mapper::MustMatch)?;

            return Ok(Parameters { params: vec![] });
        };

        let mut params = vec![arg];

        while p.r#match(Symbol::Semicolon).is_ok() {
            params.push(p.parse::<PU<Parameter>>()?);
        }

        p.r#match(Symbol::EndOfBlock).apply(mapper::MustMatch)?;
        Ok(Parameters { params })
    }
}

/// however, this is the "best" way
macro_rules! statement_wrapper {
    (
        $(
            $(#[$metas:meta])*
            $from:ident => $into:ident,
        )*
    ) => {
        $(
        #[derive(Debug, Clone)]
        $(#[$metas])*
        pub struct $into(py_lex::PU<$from>);

        impl terl::ParseUnit<py_lex::Token> for $into {
            type Target = $into;

            fn parse(p: &mut terl::Parser<py_lex::Token>) -> terl::ParseResult<Self, py_lex::Token> {

                let inner = p.parse::<py_lex::PU<$from>>()?;
                p.r#match(py_lex::syntax::Symbol::Semicolon).apply(terl::mapper::MustMatch)?;
                Ok($into(inner))
            }
        }

        impl std::ops::Deref for $into {
            type Target =  py_lex::PU<$from>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $into {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        )*
    };
}

statement_wrapper! {
    VarDefine => VarDefineStmt,
    FnCall => FnCallStmt,
    VarStore => VarStoreStmt,
}

/// be different from [`crate::complex_pu`], this version using box to make [`Statement`] enum smaller
macro_rules! statements {
    (
        $(#[$metas:meta])*
        stmt $enum_name:ident {
        $(
            $(#[$v_metas:meta])*
            $variant:ident
        ),*
    }) => {
        #[derive(Debug, Clone)]
        $(#[$metas])*
        pub enum $enum_name {
            $(
                $(#[$v_metas])*
                $variant(Box<$variant>),
            )*
        }

        $(
        impl From<$variant> for $enum_name {
             fn from(v: $variant) -> $enum_name {
                <$enum_name>::$variant(Box::new(v))
            }
        }
        )*

        impl terl::ParseUnit<py_lex::Token> for $enum_name {
            type Target = $enum_name;

            fn parse(p: &mut terl::Parser<py_lex::Token>) -> terl::ParseResult<Self, py_lex::Token>
            {
                terl::Try::<$enum_name ,_>::new(p)
                $(
                .or_try::<Self, _>(|p| {
                    p.once_no_try::<$variant ,_>($variant::parse)
                        .map(Box::new).map(<$enum_name>::$variant)
                })
                )*
                .finish()
            }
        }
    };
}

statements! {
    stmt Statement {

        // $name (...)
        FnCallStmt,
        // $name = $expr
        VarStoreStmt,

        // $ty $name
        VarDefineStmt,
        If,
        While,
        Return,
        Comment,
        CodeBlock
    }
}

complex_pu! {
    cpu Item {
        // $ty $name (...)
        FnDefine,
        Comment
    }
}

#[cfg(test)]
mod tests {
    use crate::parse_test;

    use super::*;

    #[test]
    fn variable_define() {
        parse_test("kuan1 32 zheng3 a", |p| {
            p.parse::<VarDefine>()?;
            Ok(())
        })
    }

    #[test]
    fn variable_define_init() {
        let src = "kuan1 32 zheng3 a wei2 114514 fen1";
        parse_test(src, |p| {
            p.parse::<VarDefine>()?;
            Ok(())
        });
        parse_test(src, |p| {
            p.parse::<Statement>()?;
            Ok(())
        });
    }

    #[test]
    fn variable_reassign() {
        parse_test("a wei2 114514 fen1", |p| {
            p.parse::<Statement>()?;
            Ok(())
        });
        parse_test("a wei2 114514 fen1", |p| {
            p.parse::<VarStore>()?;
            Ok(())
        });
    }
}
