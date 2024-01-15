use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum Reg {
    AX,
    DX,
    R10,
    R11,
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Reg::AX => write!(f, "%eax"),
            Reg::DX => write!(f, "%edx"),
            Reg::R10 => write!(f, "%r10d"),
            Reg::R11 => write!(f, "%r11d"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    Imm(i64),
    Reg(Reg),
    Pseudo(String),
    Stack(i64),
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Operand::Reg(ref reg) => write!(f, "{}", reg),
            Operand::Imm(i) => write!(f, "${}", i),
            Operand::Pseudo(ref name) => write!(f, "%{name}"),
            Operand::Stack(i) => write!(f, "{i}(%rbp)"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOperator {
    Neg,
    Not,
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            UnaryOperator::Neg => write!(f, "negl"),
            UnaryOperator::Not => write!(f, "notl"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mult,
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            BinaryOperator::Add => write!(f, "addl"),
            BinaryOperator::Sub => write!(f, "subl"),
            BinaryOperator::Mult => write!(f, "imull"),
        }
    }
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

impl Display for CondCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            CondCode::E => write!(f, "e"),
            CondCode::NE => write!(f, "ne"),
            CondCode::G => write!(f, "g"),
            CondCode::GE => write!(f, "ge"),
            CondCode::L => write!(f, "l"),
            CondCode::LE => write!(f, "le"),
        }
    }
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
    Ret,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Instruction::Mov(ref src, ref dst) => write!(f, "\tmovl {src}, {dst}\n"),
            Instruction::Unary(ref operator, ref dst) => write!(f, "\t{operator} {dst}\n"),
            Instruction::Binary {
                ref op,
                ref src,
                ref dst,
            } => write!(f, "\t{op} {src}, {dst}\n"),
            Instruction::Cmp(ref src, ref dst) => write!(f, "\tcmpl {src}, {dst}\n"),
            Instruction::Idiv(ref operand) => write!(f, "\tidivl {operand}\n"),
            Instruction::Cdq => write!(f, "\tcdq\n"),
            Instruction::Jmp(ref label) => write!(f, "\tjmp .L{label}\n"),
            Instruction::JmpCC(ref code, ref label) => write!(f, "\tj{code} .L{label}\n"),
            Instruction::SetCC(ref code, ref operand) => write!(f, "\tset{code} {operand}\n"),
            Instruction::Label(ref label) => write!(f, "{label}:\n"),
            Instruction::AllocateStack(i) => write!(f, "\tsubq ${i}, %rsp\n"),
            Instruction::Ret => write!(
                f,
                "
\tmovq %rbp, %rsp
\tpopq %rbp
\tret
"
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FunctionDefinition {
    Function {
        name: String,
        instructions: Vec<Instruction>,
    },
}

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            FunctionDefinition::Function {
                ref name,
                ref instructions,
            } => {
                let mut result = format!(
                    "
\t.globl {name}
{name}:
\tpushq %rbp
\tmovq %rsp, %rbp
"
                );
                for i in instructions {
                    result.push_str(format!("{}", i).as_str());
                }
                write!(f, "{result}")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Program {
    FunctionDefinition(FunctionDefinition),
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Program::FunctionDefinition(ref fn_def) => write!(
                f,
                "
{fn_def}
\t.section .note.GNU-stack,\"\",@progbits\n"
            ),
        }
    }
}
