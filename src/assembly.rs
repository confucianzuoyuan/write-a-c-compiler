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
    SP,
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
pub enum AsmType {
    Longword,
    Quadword,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    Mov(AsmType, Operand, Operand),
    Movsx(Operand, Operand),
    Unary(UnaryOperator, AsmType, Operand),
    Binary {
        op: BinaryOperator,
        t: AsmType,
        src: Operand,
        dst: Operand,
    },
    Cmp(AsmType, Operand, Operand),
    Idiv(AsmType, Operand),
    Cdq(AsmType),
    Jmp(String),
    JmpCC(CondCode, String),
    SetCC(CondCode, Operand),
    Label(String),
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
        alignment: i64,
        global: bool,
        init: i64,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum T {
    Program(Vec<TopLevel>),
}
