use crate::types;

#[derive(Clone, Debug, PartialEq)]
pub enum StaticInit {
    IntInit(i32),
    LongInit(i64),
}

pub fn zero(t: types::Type) -> StaticInit {
    match t {
        types::Type::Int => StaticInit::IntInit(0 as i32),
        types::Type::Long => StaticInit::LongInit(0 as i64),
        types::Type::FunType {
            param_types: _,
            ret_type: _,
        } => panic!("内部错误：0对于函数类型无意义。"),
    }
}

pub fn is_zero(t: StaticInit) -> bool {
    match t {
        StaticInit::IntInit(i) => i == 0 as i32,
        StaticInit::LongInit(l) => l == 0 as i64,
    }
}
