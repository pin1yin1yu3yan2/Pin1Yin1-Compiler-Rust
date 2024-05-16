pub trait IntoIR {
    type Forward;
    fn into_ir(self, map: &crate::DeclareMap) -> Self::Forward;
}

pub mod mir_variable {
    use crate::{branches, BranchesBuilder, DeclareMap, GroupIdx};
    use py_ir as ir;
    use py_ir::value::Literal;
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
        pub args: Vec<Value>,
    }

    impl From<Literal> for AtomicExpr {
        fn from(v: Literal) -> Self {
            Self::Literal(v)
        }
    }

    #[derive(Debug, Clone)]
    pub struct Value {
        pub val: AtomicExpr,
        pub ty: GroupIdx,
    }

    impl py_ir::IRValue for Value {
        type ComputeType = GroupIdx;
        type VarDefineType = GroupIdx;
        type FnDefineType = ir::types::TypeDefine;
        type ParameterType = ir::types::TypeDefine;
    }

    impl Value {
        pub fn new(val: AtomicExpr, ty: GroupIdx) -> Self {
            Value { val, ty }
        }

        pub fn is_literal(&self) -> bool {
            !matches!(self.val, AtomicExpr::FnCall(..) | AtomicExpr::Variable(..))
        }

        pub fn literal_branches(var: &Literal) -> Vec<BranchesBuilder> {
            use py_ir::types::PrimitiveType;
            match var {
                Literal::Char(_) => branches! {() =>  PrimitiveType::char()},
                // String: greatly in processing...
                Literal::Integer(_) => branches! {
                    () =>  PrimitiveType::U8, () =>  PrimitiveType::U16,
                    () =>  PrimitiveType::U32,() =>  PrimitiveType::U64,
                    () =>  PrimitiveType::U128,() =>  PrimitiveType::Usize,
                    () =>  PrimitiveType::I8, () =>  PrimitiveType::I16,
                    () =>  PrimitiveType::I32,() =>  PrimitiveType::I64,
                    () =>  PrimitiveType::I128,() =>  PrimitiveType::Isize

                },
                Literal::Float(_) => branches! {
                    () =>  PrimitiveType::F32,
                    () =>  PrimitiveType::F64
                },
            }
        }
    }

    impl IntoIR for Value {
        type Forward = ir::value::Value;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            use ir::value::*;
            match self.val {
                AtomicExpr::Literal(literal) => {
                    let primitive_type = *map.get_type(self.ty).as_primitive().unwrap();
                    Value::Literal(literal, primitive_type)
                }
                AtomicExpr::Variable(item) => Value::Variable(item),
                AtomicExpr::FnCall(item) => {
                    let unique = &map[self.ty].result();
                    let fn_name = unique.overload().name.clone();
                    let args = item.args.into_ir(map);
                    Value::FnCall(FnCall::<Value> { fn_name, args })
                }
            }
        }
    }

    impl IntoIR for Vec<Value> {
        type Forward = Vec<ir::value::Value>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            self.into_iter().map(|var| var.into_ir(map)).collect()
        }
    }
}

pub use mir_variable::*;

mod into_ir_impls {

    use super::IntoIR;
    use super::Value as MirVariable;
    use crate::DeclareMap;
    use py_ir::value::Value;
    use py_ir::*;

    impl IntoIR for Item<MirVariable> {
        type Forward = Item<Value>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            match self {
                Item::FnDefine(fn_define) => fn_define.into_ir(map).into(),
            }
        }
    }

    impl IntoIR for FnDefine<MirVariable> {
        type Forward = FnDefine<Value>;

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
        type Forward = Statements<Value>;

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
        type Forward = Statement<Value>;

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
        type Forward = OperateExpr<Value>;

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
        type Forward = Compute<Value>;

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
        type Forward = VarDefine<Value>;

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
        type Forward = VarStore<Value>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            VarStore {
                name: self.name,
                val: self.val.into_ir(map),
            }
        }
    }

    impl IntoIR for Condition<MirVariable> {
        type Forward = Condition<Value>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            Condition {
                val: self.val.into_ir(map),
                compute: self.compute.into_ir(map),
            }
        }
    }

    impl IntoIR for IfBranch<MirVariable> {
        type Forward = IfBranch<Value>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            IfBranch {
                cond: self.cond.into_ir(map),
                body: self.body.into_ir(map),
            }
        }
    }

    impl IntoIR for If<MirVariable> {
        type Forward = If<Value>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            If {
                branches: self
                    .branches
                    .into_iter()
                    .map(|branch| branch.into_ir(map))
                    .collect(),
                else_: self.else_.map(|else_| else_.into_ir(map)),
            }
        }
    }

    impl IntoIR for While<MirVariable> {
        type Forward = While<Value>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            While {
                cond: self.cond.into_ir(map),
                body: self.body.into_ir(map),
            }
        }
    }

    impl IntoIR for Return<MirVariable> {
        type Forward = Return<Value>;

        fn into_ir(self, map: &DeclareMap) -> Self::Forward {
            Return {
                val: self.val.map(|val| val.into_ir(map)),
            }
        }
    }
}

py_ir::custom_ir_variable!(pub IR<mir_variable::Value>);
