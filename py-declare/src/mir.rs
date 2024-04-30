use crate::{benches, BenchBuilder, DeclareMap, GroupIdx};
pub use py_ir::ir::{ComplexType, Literal, Parameter, Parameters, PrimitiveType, TypeDefine};
use py_ir::{
    ir::{self},
    ops::Operators,
};

pub trait IntoIR {
    type Forward;
    fn into_ir(self, map: &DeclareMap) -> Self::Forward;
}

#[derive(Default, Debug, Clone)]
pub struct Statements {
    pub stmts: Vec<Statement>,
    pub returned: bool,
}

impl Statements {
    pub fn new(stmts: Vec<Statement>, returned: bool) -> Self {
        Self { stmts, returned }
    }
}

impl From<Vec<Statement>> for Statements {
    fn from(stmts: Vec<Statement>) -> Self {
        Self {
            returned: stmts.iter().any(|stmt| stmt.returned()),
            stmts,
        }
    }
}

impl IntoIR for Statements {
    type Forward = ir::Statements;

    fn into_ir(self, map: &DeclareMap) -> Self::Forward {
        ir::Statements {
            stmts: self
                .stmts
                .into_iter()
                .map(|stmt| stmt.into_ir(map))
                .collect(),
            returned: self.returned,
        }
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

#[derive(Debug, Clone)]
pub enum Item {
    FnDefine(FnDefine),
}

impl IntoIR for Item {
    type Forward = ir::Item;

    fn into_ir(self, map: &DeclareMap) -> Self::Forward {
        match self {
            Item::FnDefine(fn_def) => fn_def.into_ir(map).into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    Compute(Compute),
    VarDefine(VarDefine),
    VarStore(VarStore),
    Block(Statements),
    FnCall(FnCallStmt),
    If(If),
    While(While),
    Return(Return),
}

impl ControlFlow for Statement {
    fn returned(&self) -> bool {
        match self {
            Statement::Block(s) => s.returned(),
            Statement::If(s) => s.returned(),
            Statement::While(s) => s.returned(),
            Statement::Return(s) => s.returned(),

            // Statement::Compute(s) => s.returned(),
            // Statement::VarDefine(s) => s.returned(),
            // Statement::VarStore(s) => s.returned(),
            // Statement::FnCall(s) => s.returned(),
            // Statement::FnDefine(s) => s.returned(),
            _ => false,
        }
    }
}

impl IntoIR for Statement {
    type Forward = ir::Statement;

    fn into_ir(self, map: &DeclareMap) -> Self::Forward {
        match self {
            Statement::Compute(item) => item.into_ir(map).into(),
            Statement::VarDefine(item) => item.into_ir(map).into(),
            Statement::VarStore(item) => item.into_ir(map).into(),
            Statement::Block(item) => item.into_ir(map).into(),
            Statement::FnCall(item) => item.into_ir(map).into(),
            Statement::If(item) => item.into_ir(map).into(),
            Statement::While(item) => item.into_ir(map).into(),
            Statement::Return(item) => item.into_ir(map).into(),
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

    impl From<FnCallStmt> for Statement {
        fn from(v: FnCallStmt) -> Self {
            Self::FnCall(v)
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

#[derive(Debug, Clone)]
pub enum AtomicExpr {
    Literal(Literal),
    Variable(String),
    // mir and ir has different fn_call define
    FnCall(FnCall),
    // #[deprecated = "unsupported now"]
    // Initialization(Vec<Expr>),
}

impl From<Literal> for AtomicExpr {
    fn from(v: Literal) -> Self {
        Self::Literal(v)
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub val: AtomicExpr,
    pub ty: GroupIdx,
}

impl IntoIR for Variable {
    type Forward = ir::Variable;

    fn into_ir(self, map: &DeclareMap) -> Self::Forward {
        match self.val {
            AtomicExpr::Literal(literal) => {
                let primitive_type = *map.get_type(self.ty).as_primitive().unwrap();
                ir::Variable::Literal(literal, primitive_type)
            }
            AtomicExpr::Variable(item) => ir::Variable::Variable(item),
            AtomicExpr::FnCall(item) => {
                let unique = &map[self.ty].unique().unwrap();
                let fn_name = unique.overload().name.clone();
                let args = item.args.into_ir(map);
                ir::Variable::FnCall(ir::FnCall { fn_name, args })
            }
        }
    }
}

impl Variable {
    pub fn new(val: AtomicExpr, ty: GroupIdx) -> Self {
        Variable { val, ty }
    }

    pub fn is_literal(&self) -> bool {
        !matches!(self.val, AtomicExpr::FnCall(..) | AtomicExpr::Variable(..))
    }

    pub fn literal_benches(var: &Literal) -> Vec<BenchBuilder> {
        match var {
            Literal::Char(_) => benches! {() => PrimitiveType::char()},
            // String: greatly in processing...
            Literal::Integer(_) => benches! {
                () => PrimitiveType::U8, () => PrimitiveType::U16,
                () => PrimitiveType::U32,() => PrimitiveType::U64,
                () => PrimitiveType::U128,() => PrimitiveType::Usize,
                () => PrimitiveType::I8, () => PrimitiveType::I16,
                () => PrimitiveType::I32,() => PrimitiveType::I64,
                () => PrimitiveType::I128,() => PrimitiveType::Isize

            },
            Literal::Float(_) => benches! {
                () => PrimitiveType::F32,
                () => PrimitiveType::F64
            },
        }
    }
}

pub type Variables = Vec<Variable>;

impl IntoIR for Variables {
    type Forward = ir::Variables;

    fn into_ir(self, map: &DeclareMap) -> Self::Forward {
        self.into_iter().map(|var| var.into_ir(map)).collect()
    }
}

/// be same as [`ir::OperateExpr`], this is also around primitive types
#[derive(Debug, Clone)]
pub enum OperateExpr {
    // type must be known, and then pick a operator-overload
    Unary(Operators, Variable),
    Binary(Operators, Variable, Variable),
}

impl IntoIR for OperateExpr {
    type Forward = ir::OperateExpr;

    fn into_ir(self, map: &DeclareMap) -> Self::Forward {
        match self {
            OperateExpr::Unary(op, l) => {
                let l = l.into_ir(map);
                ir::OperateExpr::Unary(op, l)
            }
            OperateExpr::Binary(op, l, r) => {
                let l = l.into_ir(map);
                let r = r.into_ir(map);
                ir::OperateExpr::Binary(op, l, r)
            }
        }
    }
}

impl OperateExpr {
    pub fn binary(op: Operators, l: impl Into<Variable>, r: impl Into<Variable>) -> Self {
        Self::Binary(op, l.into(), r.into())
    }

    pub fn unary(op: Operators, v: impl Into<Variable>) -> Self {
        Self::Unary(op, v.into())
    }
}

#[derive(Debug, Clone)]
pub struct Compute {
    pub ty: GroupIdx,
    pub name: String,
    pub eval: OperateExpr,
}

impl IntoIR for Compute {
    type Forward = ir::Compute;

    fn into_ir(self, map: &DeclareMap) -> Self::Forward {
        let ty = *map.get_type(self.ty).as_primitive().unwrap();
        ir::Compute {
            ty,
            eval: self.eval.into_ir(map),
            name: self.name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FnDefine {
    pub ty: TypeDefine,
    // mangled
    pub name: String,
    pub params: Parameters,
    pub body: Statements,
}

impl ControlFlow for FnDefine {
    fn returned(&self) -> bool {
        self.body.returned()
    }
}

impl IntoIR for FnDefine {
    type Forward = ir::FnDefine;

    fn into_ir(self, map: &DeclareMap) -> Self::Forward {
        ir::FnDefine {
            ty: self.ty,
            name: self.name,
            params: self.params,
            body: self.body.into_ir(map),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VarDefine {
    pub ty: TypeDefine,
    pub name: String,
    pub init: Option<Variable>,
}

impl IntoIR for VarDefine {
    type Forward = ir::VarDefine;

    fn into_ir(self, map: &DeclareMap) -> Self::Forward {
        ir::VarDefine {
            ty: self.ty,
            name: self.name,
            init: self.init.map(|init| init.into_ir(map)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VarStore {
    pub name: String,
    pub val: Variable,
}

impl IntoIR for VarStore {
    type Forward = ir::VarStore;

    fn into_ir(self, map: &DeclareMap) -> Self::Forward {
        ir::VarStore {
            name: self.name,
            val: self.val.into_ir(map),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FnCall {
    pub args: Variables,
}

#[derive(Debug, Clone)]
pub struct FnCallStmt {
    pub temp: String,
    pub called: GroupIdx,
    pub args: FnCall,
}

impl IntoIR for FnCallStmt {
    type Forward = ir::VarDefine;

    fn into_ir(self, map: &DeclareMap) -> Self::Forward {
        let unique = &map[self.called].unique().unwrap();
        VarDefine {
            ty: unique.overload().ty.clone(),
            name: self.temp,
            init: Some(Variable {
                val: AtomicExpr::FnCall(self.args),
                ty: self.called,
            }),
        }
        .into_ir(map)
    }
}

#[derive(Debug, Clone)]
pub struct Condition {
    // the final value of the condition
    pub val: Variable,
    pub compute: Statements,
}

impl IntoIR for Condition {
    type Forward = ir::Condition;

    fn into_ir(self, map: &DeclareMap) -> Self::Forward {
        ir::Condition {
            val: self.val.into_ir(map),
            compute: self.compute.into_ir(map),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IfBranch {
    pub cond: Condition,
    pub body: Statements,
}

impl ControlFlow for IfBranch {
    fn returned(&self) -> bool {
        self.body.returned()
    }
}

impl IntoIR for IfBranch {
    type Forward = ir::IfBranch;

    fn into_ir(self, map: &DeclareMap) -> Self::Forward {
        ir::IfBranch {
            cond: self.cond.into_ir(map),
            body: self.body.into_ir(map),
        }
    }
}

#[derive(Debug, Clone)]
pub struct If {
    pub branches: Vec<IfBranch>,
    pub else_: Option<Statements>,
}

impl ControlFlow for If {
    fn returned(&self) -> bool {
        self.else_.as_ref().is_some_and(|else_| else_.returned())
            && self.branches.iter().all(|branch| branch.returned())
    }
}

impl IntoIR for If {
    type Forward = ir::If;

    fn into_ir(self, map: &DeclareMap) -> Self::Forward {
        ir::If {
            branches: self
                .branches
                .into_iter()
                .map(|bench| bench.into_ir(map))
                .collect(),
            else_: self.else_.map(|else_| else_.into_ir(map)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct While {
    pub cond: Condition,
    pub body: Statements,
}

impl ControlFlow for While {
    fn returned(&self) -> bool {
        false
    }
}

impl IntoIR for While {
    type Forward = ir::While;

    fn into_ir(self, map: &DeclareMap) -> Self::Forward {
        ir::While {
            cond: self.cond.into_ir(map),
            body: self.body.into_ir(map),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Return {
    pub val: Option<Variable>,
}

impl ControlFlow for Return {
    fn returned(&self) -> bool {
        true
    }
}

impl IntoIR for Return {
    type Forward = ir::Return;

    fn into_ir(self, map: &DeclareMap) -> Self::Forward {
        ir::Return {
            val: self.val.map(|val| val.into_ir(map)),
        }
    }
}
