pub trait Translate<Item: ?Sized> {
    fn translate(&mut self, item: &Item) -> std::fmt::Result;
}

use std::fmt::Write;

use py_ir::value::Value as IRValue;

fn encode_base32(src: &str) -> String {
    base32::encode(base32::Alphabet::Crockford, src.as_bytes())
}

impl Translate<py_ir::Item<IRValue>> for crate::FileModule {
    fn translate(&mut self, item: &py_ir::Item) -> std::fmt::Result {
        match item {
            py_ir::Item::FnDefine(item) => self.translate(item),
        }
    }
}
impl Translate<py_ir::FnDefine<IRValue>> for crate::FileModule {
    fn translate(&mut self, item: &py_ir::FnDefine<IRValue>) -> std::fmt::Result {
        let write_sign = |s: &mut crate::FileModule| {
            s.translate(&item.ty)?;
            write!(s, " _{}(", encode_base32(&item.name))?;
            s.translate(&*item.params)?;
            s.write_char(')')
        };

        if item.export {
            self.write_header_file(|s| {
                write_sign(s)?;
                s.eol()
            })?;
        }

        self.write_source_file(|s| {
            write_sign(s)?;
            s.translate(&item.body)
        })
    }
}
impl Translate<py_ir::Parameter<py_ir::types::TypeDefine>> for crate::FileModule {
    fn translate(&mut self, item: &py_ir::Parameter<py_ir::types::TypeDefine>) -> std::fmt::Result {
        self.translate(&item.ty)?;
        write!(self, " {}", item.name)
    }
}
impl Translate<py_ir::Statements<IRValue>> for crate::FileModule {
    fn translate(&mut self, item: &py_ir::Statements<IRValue>) -> std::fmt::Result {
        self.write_char('{')?;
        for item in &**item {
            self.translate(item)?;
        }
        self.write_char('}')
    }
}
impl Translate<py_ir::Statement<IRValue>> for crate::FileModule {
    fn translate(&mut self, item: &py_ir::Statement<IRValue>) -> std::fmt::Result {
        match item {
            py_ir::Statement::VarDefine(item) => self.translate(item),
            py_ir::Statement::VarStore(item) => self.translate(item),
            py_ir::Statement::Block(item) => self.translate(item),
            py_ir::Statement::If(item) => self.translate(item),
            py_ir::Statement::While(item) => self.translate(item),
            py_ir::Statement::Return(item) => self.translate(item),
        }
    }
}
impl Translate<py_ir::VarDefine<IRValue>> for crate::FileModule {
    fn translate(&mut self, item: &py_ir::VarDefine<IRValue>) -> std::fmt::Result {
        self.translate(&item.ty)?;
        write!(self, " {}", item.name)?;
        if let Some(init) = &item.init {
            self.write_char('=')?;
            self.translate(init)?;
        }
        self.eol()
    }
}
impl Translate<py_ir::VarStore<IRValue>> for crate::FileModule {
    fn translate(&mut self, item: &py_ir::VarStore<IRValue>) -> std::fmt::Result {
        self.write_str(&item.name)?;
        self.write_char('=')?;
        self.translate(&item.val)?;
        self.eol()
    }
}
impl Translate<py_ir::If<IRValue>> for crate::FileModule {
    fn translate(&mut self, item: &py_ir::If<IRValue>) -> std::fmt::Result {
        let conds = (0..item.branches.len() + 1)
            .map(|_| self.label())
            .collect::<Vec<_>>();
        let codes = (0..item.branches.len() + 1)
            .map(|_| self.label())
            .collect::<Vec<_>>();

        let else_block = conds.last().unwrap();
        let code_after = codes.last().unwrap();

        for branch_idx in 0..item.branches.len() {
            self.translate(&conds[branch_idx])?;
            self.translate(&item.branches[branch_idx].cond)?;
            self.if_else(
                &item.branches[branch_idx].cond.val,
                &codes[branch_idx],
                &conds[branch_idx + 1],
            )?;

            self.translate(&codes[branch_idx])?;
            self.translate(&item.branches[branch_idx].body)?;
            self.goto(code_after)?;
        }

        self.translate(else_block)?;
        if let Some(else_) = &item.else_ {
            self.translate(else_)?;
        } else {
            self.eol()?;
        }

        self.translate(code_after)
    }
}
impl Translate<py_ir::While<IRValue>> for crate::FileModule {
    fn translate(&mut self, item: &py_ir::While<IRValue>) -> std::fmt::Result {
        let lcond = self.label();
        let lbody = self.label();
        let lafter = self.label();

        self.translate(&lcond)?;
        self.translate(&item.cond)?;
        self.if_else(&item.cond.val, &lbody, &lafter)?;

        self.translate(&lbody)?;
        self.translate(&item.body)?;
        self.goto(&lcond)?;

        self.translate(&lafter)
    }
}
impl Translate<py_ir::Condition<IRValue>> for crate::FileModule {
    fn translate(&mut self, item: &py_ir::Condition<IRValue>) -> std::fmt::Result {
        for item in &**item.compute {
            self.translate(item)?;
        }
        Ok(())
    }
}
impl Translate<crate::Label> for crate::FileModule {
    fn translate(&mut self, item: &crate::Label) -> std::fmt::Result {
        self.write_str(&item.0)?;
        self.write_char(':')?;
        self.eol()
    }
}
impl Translate<py_ir::Return<IRValue>> for crate::FileModule {
    fn translate(&mut self, item: &py_ir::Return<IRValue>) -> std::fmt::Result {
        self.write_str("return ")?;
        if let Some(val) = &item.val {
            self.translate(val)?;
        }
        self.eol()
    }
}
impl Translate<py_ir::value::AssignValue> for crate::FileModule {
    fn translate(&mut self, item: &py_ir::value::AssignValue) -> std::fmt::Result {
        match item {
            py_ir::value::AssignValue::FnCall(fn_call) => {
                write!(self, "_{}(", encode_base32(&fn_call.fn_name))?;
                self.translate(&*fn_call.args)?;
                self.write_char(')')
            }
            py_ir::value::AssignValue::Value(value) => self.translate(value),
            py_ir::value::AssignValue::Operate(op, _) => match op {
                py_ir::value::Operate::Unary(op, v) => {
                    let op = match op {
                        py_lex::ops::Operators::Not => "!",
                        py_lex::ops::Operators::Bnot => "~",
                        py_lex::ops::Operators::AddrOf => "&",
                        py_lex::ops::Operators::Deref => "*",
                        py_lex::ops::Operators::Log => {
                            self.write_str("log(")?;
                            self.translate(v)?;
                            return self.write_str(")");
                        }
                        py_lex::ops::Operators::SizeOf => {
                            self.write_str("sizeof(")?;
                            self.translate(v)?;
                            return self.write_str(")");
                        }
                        _ => panic!("unreadable or todo"),
                    };
                    self.write_str(op)?;
                    self.translate(v)
                }
                py_ir::value::Operate::Binary(op, l, r) => {
                    let op = match op {
                        py_lex::ops::Operators::Add => "+",
                        py_lex::ops::Operators::Sub => "-",
                        py_lex::ops::Operators::Mul => "*",
                        py_lex::ops::Operators::Div => "/",
                        py_lex::ops::Operators::Mod => "%",
                        py_lex::ops::Operators::Eq => "==",
                        py_lex::ops::Operators::Neq => "!=",
                        py_lex::ops::Operators::Gt => ">",
                        py_lex::ops::Operators::Lt => "<",
                        py_lex::ops::Operators::Ge => ">=",
                        py_lex::ops::Operators::Le => "<=",
                        py_lex::ops::Operators::And => "&&",
                        py_lex::ops::Operators::Or => "||",
                        py_lex::ops::Operators::Band => "&",
                        py_lex::ops::Operators::Bor => "|",
                        py_lex::ops::Operators::Xor => "^",
                        py_lex::ops::Operators::Shl => "<<",
                        py_lex::ops::Operators::Shr => ">>",
                        py_lex::ops::Operators::GetElement => ".",
                        py_lex::ops::Operators::Pow => {
                            self.write_str("pow(")?;
                            self.translate(l)?;
                            self.write_char(',')?;
                            self.translate(r)?;
                            return self.write_char(')');
                        }
                        _ => panic!("unreadable or todo"),
                    };
                    self.translate(l)?;
                    self.write_str(op)?;
                    self.translate(r)
                }
            },
        }
    }
}
impl Translate<py_ir::value::Value> for crate::FileModule {
    fn translate(&mut self, item: &py_ir::value::Value) -> std::fmt::Result {
        match item {
            IRValue::Variable(var) => self.write_str(var),
            IRValue::Literal(l, _) => write!(self, "{l}"),
        }
    }
}
impl Translate<py_ir::types::TypeDefine> for crate::FileModule {
    fn translate(&mut self, item: &py_ir::types::TypeDefine) -> std::fmt::Result {
        match item {
            py_ir::types::TypeDefine::Primitive(item) => self.translate(item),
            py_ir::types::TypeDefine::Complex(item) => self.translate(item),
        }
    }
}
impl Translate<py_ir::types::PrimitiveType> for crate::FileModule {
    fn translate(&mut self, item: &py_ir::types::PrimitiveType) -> std::fmt::Result {
        let ty = match item {
            py_ir::types::PrimitiveType::Bool => "bool",
            py_ir::types::PrimitiveType::I8 => "int8_t",
            py_ir::types::PrimitiveType::U8 => "uint8_t",
            py_ir::types::PrimitiveType::I16 => "int16_t",
            py_ir::types::PrimitiveType::U16 => "uint16_t",
            py_ir::types::PrimitiveType::I32 => "int32_t",
            py_ir::types::PrimitiveType::U32 => "uint32_t",
            py_ir::types::PrimitiveType::I64 => "int64_t",
            py_ir::types::PrimitiveType::U64 => "uint64_t",
            py_ir::types::PrimitiveType::I128 => todo!("unsuppert"),
            py_ir::types::PrimitiveType::U128 => todo!("unsuppert"),
            py_ir::types::PrimitiveType::Usize => todo!("unsuppert"),
            py_ir::types::PrimitiveType::Isize => todo!("unsuppert"),
            py_ir::types::PrimitiveType::F32 => "float",
            py_ir::types::PrimitiveType::F64 => "double",
        };
        self.write_str(ty)
    }
}
impl Translate<py_ir::types::ComplexType> for crate::FileModule {
    fn translate(&mut self, _item: &py_ir::types::ComplexType) -> std::fmt::Result {
        todo!()
    }
}
impl<Item> Translate<[Item]> for crate::FileModule
where
    Self: Translate<Item>,
{
    fn translate(&mut self, item: &[Item]) -> std::fmt::Result {
        if !item.is_empty() {
            let n = item.len();
            for arg in &item[0..n - 1] {
                self.translate(arg)?;
                self.write_char(',')?;
            }
            self.translate(&item[n - 1])?;
        }
        Ok(())
    }
}
