pub trait IntoIR {
    type Forward;
    fn into_ir(self, map: &crate::DeclareMap) -> Self::Forward;
}

pub mod mir_variable {
    use crate::{benches, BenchBuilder, DeclareMap, UndeclaredTy};
    use py_ir as ir;
    use py_ir::Literal;
    use py_lex::SharedString;

    use super::IntoIR;

    #[derive(Debug, Clone)]
    pub enum AtomicExpr {
        Literal(Literal),
        Variable(SharedString),
        FnCall(FnCall),
        // #[deprecated = "unsupported now"]
        // Initialization(Vec<Expr>),
    }

    #[derive(Debug, Clone)]
    pub struct FnCall {
        pub args: Vec<Variable>,
    }

    impl From<Literal> for AtomicExpr {
        fn from(v: Literal) -> Self {
            Self::Literal(v)
        }
    }

    #[derive(Debug, Clone)]
    pub struct Variable {
        pub val: AtomicExpr,
        pub ty: UndeclaredTy,
    }

    impl py_ir::IRVariable for Variable {
        type ComputeType = UndeclaredTy;
        type VarDefineType = UndeclaredTy;
        type FnDefineType = ir::TypeDefine;
        type ParameterType = ir::TypeDefine;
    }

    impl Variable {
        pub fn new(val: AtomicExpr, ty: UndeclaredTy) -> Self {
            Variable { val, ty }
        }

        pub fn is_literal(&self) -> bool {
            !matches!(self.val, AtomicExpr::FnCall(..) | AtomicExpr::Variable(..))
        }

        pub fn literal_benches(var: &Literal) -> Vec<BenchBuilder> {
            use py_ir::PrimitiveType;
            match var {
                Literal::Char(_) => benches! {() =>  PrimitiveType::char()},
                // String: greatly in processing...
                Literal::Integer(_) => benches! {
                    () =>  PrimitiveType::U8, () =>  PrimitiveType::U16,
                    () =>  PrimitiveType::U32,() =>  PrimitiveType::U64,
                    () =>  PrimitiveType::U128,() =>  PrimitiveType::Usize,
                    () =>  PrimitiveType::I8, () =>  PrimitiveType::I16,
                    () =>  PrimitiveType::I32,() =>  PrimitiveType::I64,
                    () =>  PrimitiveType::I128,() =>  PrimitiveType::Isize

                },
                Literal::Float(_) => benches! {
                    () =>  PrimitiveType::F32,
                    () =>  PrimitiveType::F64
                },
            }
        }
    }

    impl IntoIR for Variable {
        type Forward = ir::Variable;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            use ir::*;
            match self.val {
                AtomicExpr::Literal(literal) => {
                    let primitive_type = *map.get_type(self.ty).as_primitive().unwrap();
                    Variable::Literal(literal, primitive_type)
                }
                AtomicExpr::Variable(item) => Variable::Variable(item),
                AtomicExpr::FnCall(item) => {
                    let unique = &map[self.ty].unique().unwrap();
                    let fn_name = unique.overload().name.clone();
                    let args = item.args.into_ir(map);
                    Variable::FnCall(FnCall::<Variable> { fn_name, args })
                }
            }
        }
    }

    impl IntoIR for Vec<Variable> {
        type Forward = Vec<ir::Variable>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            self.into_iter().map(|var| var.into_ir(map)).collect()
        }
    }
}

pub use mir_variable::*;

mod into_ir_impls {

    use super::IntoIR;
    use super::Variable as MirVariable;
    use crate::DeclareMap;
    use py_ir::*;

    impl IntoIR for Item<MirVariable> {
        type Forward = Item<Variable>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            match self {
                Item::FnDefine(fn_define) => fn_define.into_ir(map).into(),
            }
        }
    }

    impl IntoIR for FnDefine<MirVariable> {
        type Forward = FnDefine<Variable>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            FnDefine {
                ty: self.ty,
                name: self.name,
                params: self.params,
                body: self.body.into_ir(map),
            }
        }
    }

    impl IntoIR for Statements<MirVariable> {
        type Forward = Statements<Variable>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            Statements {
                stmts: self
                    .stmts
                    .into_iter()
                    .map(|stmt| stmt.into_ir(map))
                    .collect(),
                returned: self.returned,
            }
        }
    }

    impl IntoIR for Statement<MirVariable> {
        type Forward = Statement<Variable>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            match self {
                Statement::Compute(item) => item.into_ir(map).into(),
                Statement::VarDefine(item) => item.into_ir(map).into(),
                Statement::VarStore(item) => item.into_ir(map).into(),
                Statement::Block(item) => item.into_ir(map).into(),
                Statement::If(item) => item.into_ir(map).into(),
                Statement::While(item) => item.into_ir(map).into(),
                Statement::Return(item) => item.into_ir(map).into(),
            }
        }
    }

    impl IntoIR for OperateExpr<MirVariable> {
        type Forward = OperateExpr<Variable>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            match self {
                OperateExpr::Unary(op, l) => {
                    let l = l.into_ir(map);
                    OperateExpr::Unary(op, l)
                }
                OperateExpr::Binary(op, l, r) => {
                    let l = l.into_ir(map);
                    let r = r.into_ir(map);
                    OperateExpr::Binary(op, l, r)
                }
            }
        }
    }

    impl IntoIR for Compute<MirVariable> {
        type Forward = Compute<Variable>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            let ty = *map.get_type(self.ty).as_primitive().unwrap();
            Compute {
                ty,
                eval: self.eval.into_ir(map),
                name: self.name,
            }
        }
    }

    impl IntoIR for VarDefine<MirVariable> {
        type Forward = VarDefine<Variable>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            let ty = map.get_type(self.ty).clone();
            VarDefine {
                ty,
                name: self.name,
                init: self.init.map(|init| init.into_ir(map)),
            }
        }
    }

    impl IntoIR for VarStore<MirVariable> {
        type Forward = VarStore<Variable>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            VarStore {
                name: self.name,
                val: self.val.into_ir(map),
            }
        }
    }

    impl IntoIR for Condition<MirVariable> {
        type Forward = Condition<Variable>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            Condition {
                val: self.val.into_ir(map),
                compute: self.compute.into_ir(map),
            }
        }
    }

    impl IntoIR for IfBranch<MirVariable> {
        type Forward = IfBranch<Variable>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            IfBranch {
                cond: self.cond.into_ir(map),
                body: self.body.into_ir(map),
            }
        }
    }

    impl IntoIR for If<MirVariable> {
        type Forward = If<Variable>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            If {
                branches: self
                    .branches
                    .into_iter()
                    .map(|bench| bench.into_ir(map))
                    .collect(),
                else_: self.else_.map(|else_| else_.into_ir(map)),
            }
        }
    }

    impl IntoIR for While<MirVariable> {
        type Forward = While<Variable>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            While {
                cond: self.cond.into_ir(map),
                body: self.body.into_ir(map),
            }
        }
    }

    impl IntoIR for Return<MirVariable> {
        type Forward = Return<Variable>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            Return {
                val: self.val.map(|val| val.into_ir(map)),
            }
        }
    }
}

py_ir::custom_ir_variable!(pub IR<mir_variable::Variable>);
