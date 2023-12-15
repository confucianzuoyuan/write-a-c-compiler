#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    Imm(i64),
    Register,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    Mov(Operand, Operand),
    Ret,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FunctionDefinition {
    Function {
        name: String,
        instructions: Vec<Instruction>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum T {
    Program {
        function_definition: FunctionDefinition,
    },
}
