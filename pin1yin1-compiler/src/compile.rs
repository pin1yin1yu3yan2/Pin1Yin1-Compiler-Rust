type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

use std::collections::HashMap;

use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    types::{AnyTypeEnum, BasicTypeEnum},
    values::AnyValueEnum,
};

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
    variables: HashMap<String, AnyValueEnum<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module: &str) -> Result<Self> {
        let module = context.create_module(module);

        // `void main()`
        let void = context.void_type();
        let no_return = void.fn_type(&[], false);
        let main = module.add_function("main", no_return, None);
        let basic_block = context.append_basic_block(main, "entry");

        let builder = context.create_builder();
        builder.position_at_end(basic_block);

        Ok(Self {
            execution_engine: module
                .create_jit_execution_engine(inkwell::OptimizationLevel::None)?,
            builder,
            context,
            module,
            variables: HashMap::new(),
        })
    }

    fn type_cast(&self, ty: &ast::TypeDefine) -> Result<BasicTypeEnum<'ctx>> {
        todo!()
    }

    fn to_value(&self, expr: &ast::Expr) -> Result<AnyValueEnum<'ctx>> {
        let r = match expr {
            ast::Expr::Char(c) => {
                let char = self.context.i32_type();
                // char -> u32
                char.const_int(*c as _, false).into()
            }
            ast::Expr::String(s) => {
                let u8 = self.context.i8_type();
                let mut bytes = vec![];
                for byte in s.bytes() {
                    bytes.push(u8.const_int(byte as _, false));
                }
                // &str -> &[u8]
                u8.const_array(&bytes).into()
            }
            ast::Expr::Integer(i) => {
                let i64 = self.context.i64_type();
                i64.const_int(*i as _, false).into()
            }
            ast::Expr::Float(f) => {
                let f64 = self.context.f64_type();
                f64.const_float(*f).into()
            }
            ast::Expr::Variable(v) => {
                let v = self.variables[v];
                if v.is_pointer_value() {
                    self.builder.build_load(v.into_pointer_value(), "")?.into()
                } else {
                    v
                }
            }

            _ => todo!("only atomic operations are allowed in VarStore"),
        };
        Ok(r)
    }
}

pub trait Compile {
    fn generate(&self, state: &mut CodeGen) -> Result<()>;
}

use pin1yin1_ast::ast;

impl Compile for ast::Statement {
    fn generate(&self, state: &mut CodeGen) -> Result<()> {
        match self {
            ast::Statement::FnDefine(f) => f.generate(state),
            ast::Statement::VarDefine(v) => v.generate(state),
            ast::Statement::VarStore(v) => v.generate(state),
            ast::Statement::FnCall(f) => f.generate(state),
            ast::Statement::If(i) => i.generate(state),
            ast::Statement::While(w) => w.generate(state),
            ast::Statement::Return(r) => r.generate(state),
            ast::Statement::Block(b) => {
                for s in b {
                    s.generate(state)?;
                }
                Ok(())
            }
        }
    }
}
impl Compile for ast::FnDefine {
    fn generate(&self, state: &mut CodeGen) -> Result<()> {
        todo!()
    }
}
impl Compile for ast::VarDefine {
    fn generate(&self, state: &mut CodeGen) -> Result<()> {
        if let Some(init) = &self.init {
            match init {
                ast::Expr::Binary(o, l, r) => {
                    let l = state.to_value(l)?;
                    let r = state.to_value(r)?;

                    let ty = l.get_type();
                    match ty {
                        AnyTypeEnum::IntType(_) => {
                            let l = l.into_int_value();
                            let r = r.into_int_value();
                            match o {
                                pin1yin1_ast::keywords::operators::Operators::Add => {
                                    let result = state.builder.build_int_add(l, r, "")?;
                                    state.variables.insert(self.name.clone(), result.into());
                                }
                                _ => todo!("other kinds of operators are not supported now~"),
                            }
                        }
                        AnyTypeEnum::FloatType(_) => todo!(),

                        _ => todo!("other kinds of types are not supported now~"),
                    }
                }
                ast::Expr::Unary(o, l) => todo!(),
                ast::Expr::Initialization(_) => todo!("unsupported"),
                // atomics
                _ => {
                    let alloca = state
                        .builder
                        .build_alloca(state.type_cast(&self.ty)?, &self.name)?;
                    let r = state.to_value(init)?;
                    state.builder.build_store(alloca, r.into_int_value())?;
                    state
                        .variables
                        .insert(self.name.clone(), AnyValueEnum::PointerValue(alloca));
                }
            }
        } else {
            let alloca = state
                .builder
                .build_alloca(state.type_cast(&self.ty)?, &self.name)?;
            state
                .variables
                .insert(self.name.clone(), AnyValueEnum::PointerValue(alloca));
        }
        todo!()
    }
}
impl Compile for ast::VarStore {
    fn generate(&self, state: &mut CodeGen) -> Result<()> {
        todo!()
    }
}
impl Compile for ast::FnCall {
    fn generate(&self, state: &mut CodeGen) -> Result<()> {
        todo!()
    }
}
impl Compile for ast::If {
    fn generate(&self, state: &mut CodeGen) -> Result<()> {
        todo!()
    }
}
impl Compile for ast::While {
    fn generate(&self, state: &mut CodeGen) -> Result<()> {
        todo!()
    }
}
impl Compile for ast::Return {
    fn generate(&self, state: &mut CodeGen) -> Result<()> {
        todo!()
    }
}
