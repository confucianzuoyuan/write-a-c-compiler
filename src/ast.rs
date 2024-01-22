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
    Conditional {
        condition: Box<Exp>,
        then_result: Box<Exp>,
        else_result: Box<Exp>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Return(Exp),
    Expression(Exp),
    If {
        condition: Exp,
        then_clause: Box<Statement>,
        else_clause: Option<Box<Statement>>,
    },
    Compound(Block),
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BlockItem {
    S(Statement),
    D(Declaration),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    Block(Vec<BlockItem>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum FunctionDefinition {
    Function { name: String, body: Block },
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
