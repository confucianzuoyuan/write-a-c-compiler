#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Int,
    FunType { param_count: usize },
}