use inkwell::{
    builder::{Builder, BuilderError},
    values::BasicValueEnum,
};
use py_ir::types::PrimitiveType;
use py_lex::ops::Operators;

pub fn binary<'ctx>(
    builder: &Builder<'ctx>,
    ty: PrimitiveType,
    op: Operators,
    l: BasicValueEnum<'ctx>,
    r: BasicValueEnum<'ctx>,
    name: &str,
) -> Result<BasicValueEnum<'ctx>, BuilderError> {
    if ty.is_integer() {
        let (l, r) = (l.into_int_value(), r.into_int_value());
        let val = match op {
            Operators::Add => builder.build_int_add(l, r, name)?,
            Operators::Sub => builder.build_int_sub(l, r, name)?,
            Operators::Mul => builder.build_int_mul(l, r, name)?,
            Operators::Div => {
                if ty.is_signed() {
                    builder.build_int_signed_div(l, r, name)?
                } else {
                    builder.build_int_unsigned_div(l, r, name)?
                }
            }
            Operators::Mod => {
                if ty.is_signed() {
                    builder.build_int_signed_rem(l, r, name)?
                } else {
                    builder.build_int_unsigned_rem(l, r, name)?
                }
            }
            // Operators::Pow => {
            //     todo!("std")
            // }
            // Operators::Log => {
            //     todo!("std")
            // }
            Operators::Eq => builder.build_int_compare(inkwell::IntPredicate::EQ, l, r, name)?,
            Operators::Neq => builder.build_int_compare(inkwell::IntPredicate::NE, l, r, name)?,
            Operators::Gt => {
                if ty.is_signed() {
                    builder.build_int_compare(inkwell::IntPredicate::SGT, l, r, name)?
                } else {
                    builder.build_int_compare(inkwell::IntPredicate::UGT, l, r, name)?
                }
            }
            Operators::Lt => {
                if ty.is_signed() {
                    builder.build_int_compare(inkwell::IntPredicate::SLT, l, r, name)?
                } else {
                    builder.build_int_compare(inkwell::IntPredicate::ULT, l, r, name)?
                }
            }
            Operators::Ge => {
                if ty.is_signed() {
                    builder.build_int_compare(inkwell::IntPredicate::SGE, l, r, name)?
                } else {
                    builder.build_int_compare(inkwell::IntPredicate::UGE, l, r, name)?
                }
            }
            Operators::Le => {
                if ty.is_signed() {
                    builder.build_int_compare(inkwell::IntPredicate::SLE, l, r, name)?
                } else {
                    builder.build_int_compare(inkwell::IntPredicate::ULE, l, r, name)?
                }
            }
            Operators::Band => builder.build_and(l, r, name)?,
            Operators::Bor => builder.build_or(l, r, name)?,
            Operators::Xor => builder.build_xor(l, r, name)?,
            Operators::Shl => builder.build_left_shift(l, r, name)?,
            Operators::Shr => builder.build_right_shift(l, r, ty.is_signed(), name)?,
            _ => unreachable!(),
        }
        .into();
        Ok(val)
    } else if ty.is_float() {
        let (l, r) = (l.into_float_value(), r.into_float_value());
        let val = match op {
            Operators::Add => builder.build_float_add(l, r, name)?.into(),
            Operators::Sub => builder.build_float_sub(l, r, name)?.into(),
            Operators::Mul => builder.build_float_mul(l, r, name)?.into(),
            Operators::Div => builder.build_float_div(l, r, name)?.into(),
            Operators::Mod => builder.build_float_rem(l, r, name)?.into(),
            Operators::Eq => builder
                .build_float_compare(inkwell::FloatPredicate::OEQ, l, r, name)?
                .into(),
            Operators::Neq => builder
                .build_float_compare(inkwell::FloatPredicate::ONE, l, r, name)?
                .into(),
            Operators::Gt => builder
                .build_float_compare(inkwell::FloatPredicate::OGT, l, r, name)?
                .into(),
            Operators::Lt => builder
                .build_float_compare(inkwell::FloatPredicate::OLT, l, r, name)?
                .into(),
            Operators::Ge => builder
                .build_float_compare(inkwell::FloatPredicate::OGE, l, r, name)?
                .into(),
            Operators::Le => builder
                .build_float_compare(inkwell::FloatPredicate::OLE, l, r, name)?
                .into(),
            _ => unreachable!(),
        };
        Ok(val)
    } else
    /* ty.is_bool()) */
    {
        let (l, r) = (l.into_int_value(), r.into_int_value());
        let val = match op {
            Operators::And => builder.build_and(l, r, name)?,
            Operators::Or => builder.build_or(l, r, name)?,

            _ => unreachable!(),
        };
        Ok(val.into())
    }
}

pub fn unary<'ctx>(
    builder: &Builder<'ctx>,
    ty: PrimitiveType,
    op: Operators,
    val: BasicValueEnum<'ctx>,
    name: &str,
) -> Result<BasicValueEnum<'ctx>, BuilderError> {
    if ty.is_integer() {
        let val = val.into_int_value();
        let val = match op {
            // Operators::Neg => builder.build_int_neg(val, name)?,
            Operators::Not => builder.build_not(val, name)?,
            _ => unreachable!(),
        }
        .into();
        Ok(val)
    } else if ty.is_float() {
        unreachable!()
    } else
    /* ty.is_bool()) */
    {
        let val = val.into_int_value();
        let val = match op {
            Operators::Not => builder.build_not(val, name)?,
            _ => unreachable!(),
        }
        .into();
        Ok(val)
    }
}
