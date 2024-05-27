pub trait IntoIR {
    type Forward;
    fn into_ir(self, map: &crate::DeclareGraph) -> Self::Forward;
}

pub mod mir_variable {
    use crate::{branches, BranchesBuilder, DeclareGraph, GroupIdx};
    use py_ir as ir;
    use py_ir::value::Literal;
    use py_lex::ops::Operators;

    use super::IntoIR;

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub enum Value {
        Literal(Literal),
        Variable(String),
    }

    impl From<Literal> for Value {
        fn from(v: Literal) -> Self {
            Self::Literal(v)
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct Undeclared<V> {
        pub val: V,
        pub ty: GroupIdx,
    }

    impl<V> Undeclared<V> {
        pub fn new(val: V, ty: GroupIdx) -> Self {
            Self { val, ty }
        }
    }

    impl Undeclared<Value> {
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

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub enum AssignValue {
        Value(Value),
        FnCall(FnCall),
        Operate(Operate),
    }

    impl From<Value> for AssignValue {
        fn from(v: Value) -> Self {
            Self::Value(v)
        }
    }

    impl From<FnCall> for AssignValue {
        fn from(v: FnCall) -> Self {
            Self::FnCall(v)
        }
    }

    impl From<Operate> for AssignValue {
        fn from(v: Operate) -> Self {
            Self::Operate(v)
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub enum Operate {
        Unary(Operators, Undeclared<Value>),
        Binary(Operators, Undeclared<Value>, Undeclared<Value>),
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct FnCall {
        pub args: Vec<Undeclared<Value>>,
    }

    impl py_ir::IRValue for Undeclared<Value> {
        type AssignValue = Undeclared<AssignValue>;
        type VarDefineType = GroupIdx;
        type FnDefineType = ir::types::TypeDefine;
        type ParameterType = ir::types::TypeDefine;
    }

    impl IntoIR for Undeclared<Value> {
        type Forward = ir::value::Value;

        fn into_ir(self, map: &crate::DeclareGraph) -> Self::Forward {
            match self.val {
                Value::Literal(literal) => {
                    let ty = *map.get_type(self.ty).as_primitive().unwrap();
                    ir::value::Value::Literal(literal, ty)
                }
                Value::Variable(variable) => ir::value::Value::Variable(variable),
            }
        }
    }

    impl IntoIR for Undeclared<AssignValue> {
        type Forward = ir::value::AssignValue;

        fn into_ir(self, map: &DeclareGraph) -> Self::Forward {
            match self.val {
                AssignValue::Value(value) => Undeclared::new(value, self.ty).into_ir(map).into(),
                AssignValue::FnCall(fn_call) => {
                    let fn_name = map[self.ty].result().overload().name.clone();
                    ir::value::FnCall {
                        fn_name,
                        args: fn_call.args.into_ir(map),
                    }
                    .into()
                }
                AssignValue::Operate(operate) => {
                    let ty = *map.get_type(self.ty).as_primitive().unwrap();
                    let operate = match operate {
                        Operate::Unary(op, l) => ir::value::Operate::Unary(op, l.into_ir(map)),
                        Operate::Binary(op, l, r) => {
                            ir::value::Operate::Binary(op, l.into_ir(map), r.into_ir(map))
                        }
                    };
                    (operate, ty).into()
                }
            }
        }
    }

    impl IntoIR for Vec<Undeclared<Value>> {
        type Forward = Vec<ir::value::Value>;

        fn into_ir(self, map: &DeclareGraph) -> Self::Forward {
            self.into_iter().map(|val| val.into_ir(map)).collect()
        }
    }
}

pub use mir_variable::*;

mod into_ir_impls {

    use super::{IntoIR, Undeclared};
    use crate::DeclareGraph;
    use py_ir::value::Value;
    use py_ir::*;

    type MirVariable = Undeclared<super::Value>;

    impl IntoIR for Item<MirVariable> {
        type Forward = Item<Value>;

        fn into_ir(self, map: &DeclareGraph) -> Self::Forward {
            match self {
                Item::FnDefine(fn_define) => fn_define.into_ir(map).into(),
            }
        }
    }

    impl IntoIR for FnDefine<MirVariable> {
        type Forward = FnDefine<Value>;

        fn into_ir(self, map: &DeclareGraph) -> Self::Forward {
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

        fn into_ir(self, map: &DeclareGraph) -> Self::Forward {
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

        fn into_ir(self, map: &DeclareGraph) -> Self::Forward {
            match self {
                Statement::VarDefine(item) => item.into_ir(map).into(),
                Statement::VarStore(item) => item.into_ir(map).into(),
                Statement::Block(item) => item.into_ir(map).into(),
                Statement::If(item) => item.into_ir(map).into(),
                Statement::While(item) => item.into_ir(map).into(),
                Statement::Return(item) => item.into_ir(map).into(),
            }
        }
    }

    impl IntoIR for VarDefine<MirVariable> {
        type Forward = VarDefine<Value>;

        fn into_ir(self, map: &DeclareGraph) -> Self::Forward {
            let ty = map.get_type(self.ty).clone();
            VarDefine {
                ty,
                name: self.name,
                init: self.init.map(|init| init.into_ir(map)),
                is_temp: self.is_temp,
            }
        }
    }

    impl IntoIR for VarStore<MirVariable> {
        type Forward = VarStore<Value>;

        fn into_ir(self, map: &DeclareGraph) -> Self::Forward {
            VarStore {
                name: self.name,
                val: self.val.into_ir(map),
            }
        }
    }

    impl IntoIR for Condition<MirVariable> {
        type Forward = Condition<Value>;

        fn into_ir(self, map: &DeclareGraph) -> Self::Forward {
            Condition {
                val: self.val.into_ir(map),
                compute: self.compute.into_ir(map),
            }
        }
    }

    impl IntoIR for IfBranch<MirVariable> {
        type Forward = IfBranch<Value>;

        fn into_ir(self, map: &DeclareGraph) -> Self::Forward {
            IfBranch {
                cond: self.cond.into_ir(map),
                body: self.body.into_ir(map),
            }
        }
    }

    impl IntoIR for If<MirVariable> {
        type Forward = If<Value>;

        fn into_ir(self, map: &DeclareGraph) -> Self::Forward {
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

        fn into_ir(self, map: &DeclareGraph) -> Self::Forward {
            While {
                cond: self.cond.into_ir(map),
                body: self.body.into_ir(map),
            }
        }
    }

    impl IntoIR for Return<MirVariable> {
        type Forward = Return<Value>;

        fn into_ir(self, map: &DeclareGraph) -> Self::Forward {
            Return {
                val: self.val.map(|val| val.into_ir(map)),
            }
        }
    }
}

py_ir::custom_ir_variable!(pub IR<Undeclared<mir_variable::Value>>);
