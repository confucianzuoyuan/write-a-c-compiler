use crate::{constants, types};

pub fn const_convert(target_type: types::Type, c: constants::T) -> constants::T {
    match c {
        constants::T::ConstInt(i) => {
            if target_type == types::Type::Int {
                constants::T::ConstInt(i)
            } else {
                constants::T::ConstLong(i as i64)
            }
        }
        constants::T::ConstLong(i) => {
            if target_type == types::Type::Long {
                constants::T::ConstLong(i)
            } else {
                constants::T::ConstInt(i as i32)
            }
        }
    }
}
