#[derive(Clone, Debug, PartialEq)]
pub enum T {
    ConstInt(i32),
    ConstLong(i64),
}

pub const INT_ZERO: T = T::ConstInt(0 as i32);
pub const INT_ONE: T = T::ConstInt(1 as i32);
