#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Int,
    Long,
    FunType {
        param_types: Vec<Box<Type>>,
        ret_type: Box<Type>,
    },
}
