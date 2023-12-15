use crate::assembly;
use crate::ast;

fn convert_exp(i: ast::Exp) -> assembly::Operand {
    match i {
        ast::Exp::Constant { value } => assembly::Operand::Imm(value),
    }
}

fn convert_statement(e: ast::Statement) -> Vec<assembly::Instruction> {
    match e {
        ast::Statement::Return { exp } => {
            let v = convert_exp(exp);
            vec![
                assembly::Instruction::Mov(v, assembly::Operand::Register),
                assembly::Instruction::Ret,
            ]
        }
    }
}

fn convert_function(f: ast::FunctionDefinition) -> assembly::FunctionDefinition {
    match f {
        ast::FunctionDefinition::Function { name, body } => {
            assembly::FunctionDefinition::Function {
                name: name,
                instructions: convert_statement(body),
            }
        }
    }
}

pub fn gen(fn_def: ast::T) -> assembly::T {
    match fn_def {
        ast::T::Program {
            function_definition,
        } => assembly::T::Program {
            function_definition: convert_function(function_definition),
        },
    }
}
