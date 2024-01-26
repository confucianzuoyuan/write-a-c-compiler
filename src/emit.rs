use crate::assembly;

fn show_reg(r: assembly::Reg) -> String {
    match r {
        assembly::Reg::AX => "%eax".to_string(),
        assembly::Reg::CX => "%ecx".to_string(),
        assembly::Reg::DX => "%edx".to_string(),
        assembly::Reg::DI => "%edi".to_string(),
        assembly::Reg::SI => "%esi".to_string(),
        assembly::Reg::R8 => "%r8d".to_string(),
        assembly::Reg::R9 => "%r9d".to_string(),
        assembly::Reg::R10 => "%r10d".to_string(),
        assembly::Reg::R11 => "%r11d".to_string(),
    }
}

fn show_operand(operand: assembly::Operand) -> String {
    match operand {
        assembly::Operand::Reg(r) => show_reg(r),
        assembly::Operand::Imm(i) => format!("${}", i),
        assembly::Operand::Stack(i) => format!("{}(%rbp)", i),
        assembly::Operand::Pseudo(name) => format!("%{}", name),
    }
}

fn show_byte_reg(r: assembly::Reg) -> String {
    match r {
        assembly::Reg::AX => "%al".to_string(),
        assembly::Reg::CX => "%cl".to_string(),
        assembly::Reg::DX => "%dl".to_string(),
        assembly::Reg::DI => "%dil".to_string(),
        assembly::Reg::SI => "%sil".to_string(),
        assembly::Reg::R8 => "%r8b".to_string(),
        assembly::Reg::R9 => "%r9b".to_string(),
        assembly::Reg::R10 => "r10b".to_string(),
        assembly::Reg::R11 => "r11b".to_string(),
    }
}

fn show_byte_operand(operand: assembly::Operand) -> String {
    match operand {
        assembly::Operand::Reg(r) => show_byte_reg(r),
        other => show_operand(other),
    }
}

fn show_quadword_reg(r: assembly::Reg) -> String {
    match r {
        assembly::Reg::AX => "%rax".to_string(),
        assembly::Reg::CX => "%rcx".to_string(),
        assembly::Reg::DX => "%rdx".to_string(),
        assembly::Reg::DI => "%rdi".to_string(),
        assembly::Reg::SI => "%rsi".to_string(),
        assembly::Reg::R8 => "%r8".to_string(),
        assembly::Reg::R9 => "%r9".to_string(),
        assembly::Reg::R10 => "r10".to_string(),
        assembly::Reg::R11 => "r11".to_string(),
    }
}

fn show_quadword_operand(operand: assembly::Operand) -> String {
    match operand {
        assembly::Operand::Reg(r) => show_quadword_reg(r),
        other => show_operand(other),
    }
}

fn show_label(name: String) -> String {
    name
}

fn show_fun_name(f: String) -> String {
    f
}

fn show_local_label(label: String) -> String {
    format!(".L{}", label)
}

fn show_unary_instruction(op: assembly::UnaryOperator) -> String {
    match op {
        assembly::UnaryOperator::Neg => "negl".to_string(),
        assembly::UnaryOperator::Not => "notl".to_string(),
    }
}

fn show_binary_instruction(op: assembly::BinaryOperator) -> String {
    match op {
        assembly::BinaryOperator::Add => "addl".to_string(),
        assembly::BinaryOperator::Mult => "imull".to_string(),
        assembly::BinaryOperator::Sub => "subl".to_string(),
    }
}

fn show_cond_code(code: assembly::CondCode) -> String {
    match code {
        assembly::CondCode::E => "e".to_string(),
        assembly::CondCode::G => "g".to_string(),
        assembly::CondCode::GE => "ge".to_string(),
        assembly::CondCode::L => "l".to_string(),
        assembly::CondCode::LE => "le".to_string(),
        assembly::CondCode::NE => "ne".to_string(),
    }
}

fn emit_instruction(instruction: assembly::Instruction) -> String {
    match instruction {
        assembly::Instruction::Mov(src, dst) => {
            format!("\tmovl {}, {}\n", show_operand(src), show_operand(dst))
        }
        assembly::Instruction::Unary(operator, dst) => {
            format!(
                "\t{} {}\n",
                show_unary_instruction(operator),
                show_operand(dst)
            )
        }
        assembly::Instruction::Binary { op, src, dst } => {
            format!(
                "\t{} {}, {}\n",
                show_binary_instruction(op),
                show_operand(src),
                show_operand(dst)
            )
        }
        assembly::Instruction::Cmp(src, dst) => {
            format!("\tcmpl {}, {}\n", show_operand(src), show_operand(dst))
        }
        assembly::Instruction::Idiv(operand) => {
            format!("\tidivl {}\n", show_operand(operand))
        }
        assembly::Instruction::Cdq => "\tcdq\n".to_string(),
        assembly::Instruction::Jmp(lbl) => {
            format!("\tjmp {}\n", show_local_label(lbl))
        }
        assembly::Instruction::JmpCC(code, lbl) => {
            format!("\tj{} {}\n", show_cond_code(code), show_local_label(lbl))
        }
        assembly::Instruction::SetCC(code, operand) => {
            format!(
                "\tset{} {}\n",
                show_cond_code(code),
                show_byte_operand(operand)
            )
        }
        assembly::Instruction::Label(lbl) => {
            format!("{}:\n", show_local_label(lbl))
        }
        assembly::Instruction::AllocateStack(i) => {
            format!("\tsubq ${}, %rsp\n", i)
        }
        assembly::Instruction::DeallocateStack(i) => {
            format!("\taddq ${}, %rsp\n", i)
        }
        assembly::Instruction::Push(op) => {
            format!("\tpushq {}\n", show_quadword_operand(op))
        }
        assembly::Instruction::Call(f) => {
            format!("\tcall {}\n", show_fun_name(f))
        }
        assembly::Instruction::Ret => "
\tmovq %rbp, %rsp
\tpopq %rbp
\tret
"
        .to_string(),
    }
}

fn emit_function(f: assembly::FunctionDefinition) -> String {
    match f {
        assembly::FunctionDefinition::Function { name, instructions } => {
            let label = show_label(name);
            let mut result = format!("
\t.globl {}
{}:
\tpushq %rbp
\tmovq %rsp, %rbp
", label, label);
            for instruction in instructions {
                result.push_str(&emit_instruction(instruction));
            }
            result
        }
    }
}

fn emit_stack_note() -> String {
    "\t.section .note.GNU-stack,\"\",@progbits\n".to_string()
}

pub fn emit(program: assembly::Program) {
    match program {
        assembly::Program::FunctionDefinition(fn_defs) => {
            for fn_def in fn_defs {
                print!("{}", emit_function(fn_def));
            }
            println!();
            print!("{}", emit_stack_note());
        }
    }
}