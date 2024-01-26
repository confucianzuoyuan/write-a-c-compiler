use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOperator {
    Complement,
    Negate,
    Not,
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            UnaryOperator::Complement => write!(f, "~"),
            UnaryOperator::Negate => write!(f, "-"),
            UnaryOperator::Not => write!(f, "!"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Subtract => write!(f, "-"),
            BinaryOperator::Multiply => write!(f, "*"),
            BinaryOperator::Divide => write!(f, "/"),
            BinaryOperator::Mod => write!(f, "%"),
            BinaryOperator::Equal => write!(f, "=="),
            BinaryOperator::NotEqual => write!(f, "!="),
            BinaryOperator::LessThan => write!(f, "<"),
            BinaryOperator::LessOrEqual => write!(f, "<="),
            BinaryOperator::GreaterThan => write!(f, ">"),
            BinaryOperator::GreaterOrEqual => write!(f, ">="),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum IrValue {
    Constant(i64),
    Var(String),
}

impl Display for IrValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            IrValue::Constant(c) => write!(f, "{}", c),
            IrValue::Var(ref v) => write!(f, "{}", v),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    Return(IrValue),
    Unary {
        op: UnaryOperator,
        src: IrValue,
        dst: IrValue,
    },
    Binary {
        op: BinaryOperator,
        src1: IrValue,
        src2: IrValue,
        dst: IrValue,
    },
    Copy {
        src: IrValue,
        dst: IrValue,
    },
    Jump(String),
    JumpIfZero(IrValue, String),
    JumpIfNotZero(IrValue, String),
    Label(String),
    FunCall {
        f: String,
        args: Vec<IrValue>,
        dst: IrValue,
    },
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Return(ref ir_value) => write!(f, "Return({})", ir_value),
            Instruction::Unary {
                ref op,
                ref src,
                ref dst,
            } => write!(f, "{}={}{}", dst, op, src),
            Instruction::Binary {
                ref op,
                ref src1,
                ref src2,
                ref dst,
            } => write!(f, "{}={}{}{}", dst, src1, op, src2),
            Instruction::Copy { ref src, ref dst } => write!(f, "{}={}", dst, src),
            Instruction::Jump(ref s) => write!(f, "Jump({})", s),
            Instruction::JumpIfZero(ref cond, ref target) => {
                write!(f, "JumpIfZero({}, {})", cond, target)
            }
            Instruction::JumpIfNotZero(ref cond, ref target) => {
                write!(f, "JumpIfNotZero({}, {})", cond, target)
            }
            Instruction::Label(ref label) => write!(f, "{}:", label),
            Instruction::FunCall {
                f: fun_name,
                args,
                dst,
            } => {
                let mut result = String::new();
                result.push_str(format!("{} = {}(", dst, fun_name).as_str());
                for (i, arg) in args.iter().enumerate() {
                    if i < args.len() - 1 {
                        result.push_str(format!("{}, ", arg).as_str());
                    } else {
                        result.push_str(format!("{})", arg).as_str());
                    }
                }
                write!(f, "{}", result)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FunctionDefinition {
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Instruction>,
    },
}

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionDefinition::Function {
                ref name,
                params,
                ref body,
            } => {
                let mut result = String::new();
                result.push_str(format!("{}(", name).as_str());
                for (i, param) in params.iter().enumerate() {
                    if i < params.len() - 1 {
                        result.push_str(format!("{}, ", param).as_str());
                    } else {
                        result.push_str(format!("{}):\n", param).as_str());
                    }
                }
                if params.len() == 0 {
                    result.push_str(format!("):\n").as_str());
                }
                for i in body {
                    result.push_str(format!("{}\n", i).as_str());
                }
                write!(f, "{}", result)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Program {
    FunctionDefinition(Vec<FunctionDefinition>),
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Program::FunctionDefinition(ref fn_defs) => {
                let mut result = String::new();
                for fn_def in fn_defs {
                    result.push_str(format!("{}\n", fn_def).as_str());
                }
                write!(f, "{}", result)
            }
        }
    }
}
