use crate::{ast, types};

pub fn get_type(t: ast::TypedExp) -> types::Type {
    t.t
}

pub fn set_type(e: ast::Exp, new_type: types::Type) -> ast::TypedExp {
    ast::TypedExp { e: e, t: new_type }
}

pub fn get_alignment(t: types::Type) -> i64 {
    match t {
        types::Type::Int => 4,
        types::Type::Long => 8,
        types::Type::FunType {
            param_types: _,
            ret_type: _,
        } => panic!("内部错误：函数类型不存在对齐这一说。"),
    }
}
