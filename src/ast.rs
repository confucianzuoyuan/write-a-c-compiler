use crate::{constants, types};

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOperator {
    Complement,
    Negate,
    Not,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
    And,
    Or,
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement<ExpType> {
    Return(ExpType),
    Expression(ExpType),
    If {
        condition: ExpType,
        then_clause: Box<Statement<ExpType>>,
        else_clause: Option<Box<Statement<ExpType>>>,
    },
    Compound(Block<ExpType>),
    Break(String),
    Continue(String),
    While {
        condition: ExpType,
        body: Box<Statement<ExpType>>,
        id: String,
    },
    DoWhile {
        body: Box<Statement<ExpType>>,
        condition: ExpType,
        id: String,
    },
    For {
        init: ForInit<ExpType>,
        condition: Option<ExpType>,
        post: Option<ExpType>,
        body: Box<Statement<ExpType>>,
        id: String,
    },
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BlockItem<ExpType> {
    S(Statement<ExpType>),
    D(Declaration<ExpType>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Block<ExpType> {
    Block(Vec<BlockItem<ExpType>>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProgType<ExpType> {
    Program(Vec<Declaration<ExpType>>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum StorageClass {
    Static,
    Extern,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariableDeclaration<ExpType> {
    pub name: String,
    pub var_type: types::Type,
    pub init: Option<ExpType>,
    pub storage_class: Option<StorageClass>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ForInit<ExpType> {
    InitDecl(VariableDeclaration<ExpType>),
    InitExp(Option<ExpType>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDeclaration<ExpType> {
    pub name: String,
    pub fun_type: types::Type,
    pub params: Vec<String>,
    pub body: Option<Block<ExpType>>,
    pub storage_class: Option<StorageClass>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Declaration<ExpType> {
    FunDecl(FunctionDeclaration<ExpType>),
    VarDecl(VariableDeclaration<ExpType>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnTypedExp {
    Constant(constants::T),
    Cast {
        target_type: types::Type,
        e: Box<UnTypedExp>,
    },
    Unary(UnaryOperator, Box<UnTypedExp>),
    Binary(BinaryOperator, Box<UnTypedExp>, Box<UnTypedExp>),
    Var(String),
    Assignment(Box<UnTypedExp>, Box<UnTypedExp>),
    Conditional {
        condition: Box<UnTypedExp>,
        then_result: Box<UnTypedExp>,
        else_result: Box<UnTypedExp>,
    },
    FunCall {
        f: String,
        args: Vec<UnTypedExp>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypedInnerExp {
    Constant(constants::T),
    Var(String),
    Cast {
        target_type: types::Type,
        e: TypedExp,
    },
    Unary(UnaryOperator, TypedExp),
    Binary(BinaryOperator, TypedExp, TypedExp),
    Assignment(TypedExp, TypedExp),
    Conditional {
        condition: TypedExp,
        then_result: TypedExp,
        else_result: TypedExp,
    },
    Funcall {
        f: String,
        args: Vec<TypedExp>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypedExp {
    pub e: Box<TypedInnerExp>,
    pub t: types::Type,
}
