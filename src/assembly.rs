#[derive(Clone, Debug, PartialEq)]
pub enum Reg {
    AX,
    CX,
    DX,
    DI,
    SI,
    R8,
    R9,
    R10,
    R11,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    Imm(i64),
    Reg(Reg),
    Pseudo(String),
    Stack(i64),
    Data(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOperator {
    Neg,
    Not,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mult,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CondCode {
    E,
    NE,
    G,
    GE,
    L,
    LE,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    Mov(Operand, Operand),
    Unary(UnaryOperator, Operand),
    Binary {
        op: BinaryOperator,
        src: Operand,
        dst: Operand,
    },
    Cmp(Operand, Operand),
    Idiv(Operand),
    Cdq,
    Jmp(String),
    JmpCC(CondCode, String),
    SetCC(CondCode, Operand),
    Label(String),
    AllocateStack(i64),
    DeallocateStack(i64),
    Push(Operand),
    Call(String),
    Ret,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TopLevel {
    Function {
        name: String,
        global: bool,
        instructions: Vec<Instruction>,
    },
    StaticVariable {
        name: String,
        global: bool,
        init: i64,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum T {
    Program(Vec<TopLevel>),
}
