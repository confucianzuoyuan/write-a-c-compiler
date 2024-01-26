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
    FunCall {
        f: String,
        args: Vec<Exp>,
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
    Break(String),
    Continue(String),
    While {
        condition: Exp,
        body: Box<Statement>,
        id: String,
    },
    DoWhile {
        body: Box<Statement>,
        condition: Exp,
        id: String,
    },
    For {
        init: ForInit,
        condition: Option<Exp>,
        post: Option<Exp>,
        body: Box<Statement>,
        id: String,
    },
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
pub enum Program {
    FunctionDefinition(Vec<FunctionDeclaration>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariableDeclaration {
    pub name: String,
    pub init: Option<Exp>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ForInit {
    InitDecl(VariableDeclaration),
    InitExp(Option<Exp>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDeclaration {
    pub name: String,
    pub params: Vec<String>,
    pub body: Option<Block>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Declaration {
    FunDecl(FunctionDeclaration),
    VarDecl(VariableDeclaration),
}
