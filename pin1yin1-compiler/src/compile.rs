type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;



use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    types::BasicTypeEnum,
    values::BasicValueEnum,
};

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
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
        })
    }

    fn type_cast(&self, ty: &ast::TypeDefine) -> Result<BasicTypeEnum> {
        todo!()
    }

    fn to_value(&self, expr: &ast::Expr) -> BasicValueEnum {
        match expr {
            ast::Expr::Char(c) => {
                let char = self.context.i32_type();
                // char -> u32
                char.const_int(*c as _,false).into()
                
            },
            ast::Expr::String(s) => {
                let u8 = self.context.i8_type();
                let mut bytes= vec![];
                for byte in s.bytes(){
                    bytes.push(u8.const_int(byte as _,false));
                }
                // &str -> &[u8]
                u8.const_array(&bytes).into()
            },
            ast::Expr::Integer(i) => {
                todo!()
            },
            ast::Expr::Float(_) => todo!(),
            ast::Expr::Variable(_) => todo!(),
            _ => todo!("ast should not contain these kinds of value! / you cant use these kinds of values to init a variable, etc"),
        }
    }
}

pub trait Compile {
    fn generate(&self, state: &CodeGen) -> Result<()>;
}

use pin1yin1_ast::ast;

impl Compile for ast::Statement {
    fn generate(&self, state: &CodeGen) -> Result<()> {
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
    fn generate(&self, state: &CodeGen) -> Result<()> {
        todo!()
    }
}
impl Compile for ast::VarDefine {
    fn generate(&self, state: &CodeGen) -> Result<()> {
        let alloca = state
            .builder
            .build_alloca(state.type_cast(&self.ty)?, &self.name)?;
        if let Some(init) = &self.init {
            state.builder.build_store(alloca, state.to_value(init))?;
        }
        todo!()
    }
}
impl Compile for ast::VarStore {
    fn generate(&self, state: &CodeGen) -> Result<()> {
        todo!()
    }
}
impl Compile for ast::FnCall {
    fn generate(&self, state: &CodeGen) -> Result<()> {
        todo!()
    }
}
impl Compile for ast::If {
    fn generate(&self, state: &CodeGen) -> Result<()> {
        todo!()
    }
}
impl Compile for ast::While {
    fn generate(&self, state: &CodeGen) -> Result<()> {
        todo!()
    }
}
impl Compile for ast::Return {
    fn generate(&self, state: &CodeGen) -> Result<()> {
        todo!()
    }
}
