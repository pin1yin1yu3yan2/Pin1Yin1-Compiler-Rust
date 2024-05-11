use super::*;

use py_lex::syntax::Symbol;

#[derive(Debug, Clone)]
pub struct VarAssign {
    pub val: Expr,
}

impl ParseUnit<Token> for VarAssign {
    type Target = VarAssign;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.match_(Symbol::Assign)?;
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

impl ParseUnit<Token> for Parameters {
    type Target = Parameters;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.match_(Symbol::Parameter)?;
        let Some(arg) = p.parse::<PU<Parameter>>().apply(mapper::Try)? else {
            p.match_(Symbol::EndOfBlock).apply(mapper::MustMatch)?;

            return Ok(Parameters { params: vec![] });
        };

        let mut params = vec![arg];

        while p.match_(Symbol::Semicolon).is_ok() {
            params.push(p.parse::<PU<Parameter>>()?);
        }

        p.match_(Symbol::EndOfBlock).apply(mapper::MustMatch)?;
        Ok(Parameters { params })
    }
}

#[cfg(test)]
mod tests {
    use crate::parse_test;

    use super::*;

    #[test]
    fn variable_define() {
        parse_test("kuan1 32 zheng3 a", |p| {
            assert!(p.parse::<VarDefine>().is_ok())
        })
    }

    #[test]
    fn variable_define_init() {
        let src = "kuan1 32 zheng3 a wei2 114514 fen1";
        parse_test(src, |p| assert!(p.parse::<VarDefine>().is_ok()));
        parse_test(src, |p| {
            assert!(p.parse::<Statement>().is_ok());
        });
    }

    #[test]
    fn variable_reassign() {
        parse_test("a wei2 114514 fen1", |p| {
            assert!(p.parse::<Statement>().is_ok())
        });
        parse_test("a wei2 114514 fen1", |p| {
            assert!(p.parse::<VarStore>().is_ok())
        });
    }
}
