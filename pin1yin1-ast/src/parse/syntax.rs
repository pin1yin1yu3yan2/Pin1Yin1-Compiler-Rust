use super::*;

use crate::lex::syntax::Symbol;

#[derive(Debug, Clone)]
pub struct VarAssign {
    pub deng3: PU<Symbol>,
    pub val: PU<Expr>,
}

impl ParseUnit for VarAssign {
    type Target = VarAssign;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let deng3 = p.match_(Symbol::Assign)?;
        let val = p.parse()?;
        p.finish(Self { deng3, val })
    }
}

#[derive(Debug, Clone)]
pub struct VarDefine {
    pub ty: PU<types::TypeDefine>,
    pub name: PU<Ident>,
    pub init: Option<PU<VarAssign>>,
}

impl ParseUnit for VarDefine {
    type Target = VarDefine;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let ty = p.parse()?;
        let name = p.parse()?;
        let init = p.parse().r#try()?;
        p.finish(Self { ty, name, init })
    }
}

#[derive(Debug, Clone)]
pub struct VarStore {
    pub name: PU<Ident>,
    pub assign: PU<VarAssign>,
}

impl ParseUnit for VarStore {
    type Target = VarStore;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let name = p.parse::<Ident>()?;
        let assign = p.parse::<VarAssign>()?;
        p.finish(VarStore { name, assign })
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

impl ParseUnit for Parameter {
    type Target = Parameter;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let ty = p.parse::<types::TypeDefine>()?;
        let name = p.parse::<Ident>()?;
        let inner = VarDefine {
            ty,
            name,
            init: None,
        };
        p.finish(Parameter { inner })
    }
}

#[derive(Debug, Clone)]
pub struct Parameters {
    pub params: Vec<PU<Parameter>>,
    pub semicolons: Vec<PU<Symbol>>,
}

impl From<Vec<PU<Parameter>>> for Parameters {
    fn from(value: Vec<PU<Parameter>>) -> Self {
        Self {
            params: value,
            semicolons: Vec::new(),
        }
    }
}

impl From<Parameters> for Vec<PU<Parameter>> {
    fn from(value: Parameters) -> Self {
        value.params
    }
}

impl ParseUnit for Parameters {
    type Target = Parameters;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let Some(arg) = p.parse::<Parameter>().r#try()? else {
            return p.finish(Parameters {
                params: vec![],
                semicolons: vec![],
            });
        };

        let mut params = vec![arg];
        let mut semicolons = vec![];

        while let Some(semicolon) = p.match_(Symbol::Semicolon).r#try()? {
            semicolons.push(semicolon);
            params.push(p.parse::<Parameter>()?);
        }

        p.finish(Parameters { params, semicolons })
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
        parse_test("kuan1 32 zheng3 a deng3 114514 fen1", |p| {
            assert!(p.parse::<Statement>().is_ok())
        });
        parse_test("kuan1 32 zheng3 a deng3 114514 fen1", |p| {
            assert!(p.parse::<VarDefine>().is_ok())
        });
    }

    #[test]
    fn variable_reassign() {
        parse_test("a deng3 114514 fen1", |p| {
            assert!(p.parse::<Statement>().is_ok())
        });
        parse_test("a deng3 114514 fen1", |p| {
            assert!(p.parse::<VarStore>().is_ok())
        });
    }
}
