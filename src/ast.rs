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
pub enum Exp {
    Constant(i64),
    Unary(UnaryOperator, Box<Exp>),
    Binary(BinaryOperator, Box<Exp>, Box<Exp>),
    Var(String),
    Assignment(Box<Exp>, Box<Exp>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Return(Exp),
    Expression(Exp),
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BlockItem {
    S(Statement),
    D(Declaration),
}

#[derive(Clone, Debug, PartialEq)]
pub enum FunctionDefinition {
    Function { name: String, body: Vec<BlockItem> },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Program {
    FunctionDefinition(FunctionDefinition),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub init: Option<Exp>,
}
