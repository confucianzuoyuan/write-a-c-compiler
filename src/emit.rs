use crate::assembly;

fn show_operand(operand: assembly::Operand) -> String {
    match operand {
        assembly::Operand::Register => "a0".to_string(),
        assembly::Operand::Imm(i) => format!("{}", i),
    }
}

fn emit_instruction(instruction: assembly::Instruction) {
    match instruction {
        assembly::Instruction::Mov(src, dst) => {
            println!("\tli {}, {}", show_operand(dst), show_operand(src));
        }
        assembly::Instruction::Ret => {
            println!("\tret");
        }
    }
}

fn emit_function(f: assembly::FunctionDefinition) {
    match f {
        assembly::FunctionDefinition::Function { name, instructions } => {
            println!("  .global {}", name);
            println!("{}:", name);
            for instruction in instructions {
                emit_instruction(instruction);
            }
        }
    }
}

fn emit_stack_note() {
    println!("\t.section .note.GNU-stack,\"\",@progbits");
}

pub fn emit(program: assembly::T) {
    match program {
        assembly::T::Program { function_definition } => {
            emit_function(function_definition);
            emit_stack_note();
        }
    }
    println!();
}