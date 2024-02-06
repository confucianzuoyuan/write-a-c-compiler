use crate::{assembly, initializers, symbols};

fn suffix(t: assembly::AsmType) -> String {
    match t {
        assembly::AsmType::Longword => "l".to_string(),
        assembly::AsmType::Quadword => "q".to_string(),
    }
}

fn align_directive() -> String {
    ".align".to_string()
}

fn show_label(name: String) -> String {
    name
}

fn show_local_label(label: String) -> String {
    format!(".L{}", label)
}

fn show_fun_name(f: String) -> String {
    if symbols::is_defined(f.clone()) {
        f
    } else {
        format!("{}@PLT", f)
    }
}

fn show_long_reg(r: assembly::Reg) -> String {
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
        assembly::Reg::SP => panic!("内部错误：没有32位的RSP"),
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
        assembly::Reg::R10 => "%r10".to_string(),
        assembly::Reg::R11 => "%r11".to_string(),
        assembly::Reg::SP => "rsp".to_string(),
    }
}

fn show_operand(t: assembly::AsmType, operand: assembly::Operand) -> String {
    match operand {
        assembly::Operand::Reg(r) => match t {
            assembly::AsmType::Longword => show_long_reg(r),
            assembly::AsmType::Quadword => show_quadword_reg(r),
        },
        assembly::Operand::Imm(i) => format!("${}", i),
        assembly::Operand::Stack(i) => format!("{}(%rbp)", i),
        assembly::Operand::Data(name) => format!("{}(%rip)", show_label(name)),
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
        assembly::Reg::SP => panic!("内部错误：没有一个字节的RSP寄存器。"),
    }
}

fn show_byte_operand(operand: assembly::Operand) -> String {
    match operand {
        assembly::Operand::Reg(r) => show_byte_reg(r),
        other => show_operand(assembly::AsmType::Longword, other),
    }
}

fn show_unary_instruction(op: assembly::UnaryOperator) -> String {
    match op {
        assembly::UnaryOperator::Neg => "neg".to_string(),
        assembly::UnaryOperator::Not => "not".to_string(),
    }
}

fn show_binary_instruction(op: assembly::BinaryOperator) -> String {
    match op {
        assembly::BinaryOperator::Add => "add".to_string(),
        assembly::BinaryOperator::Mult => "imul".to_string(),
        assembly::BinaryOperator::Sub => "sub".to_string(),
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
        assembly::Instruction::Mov(t, src, dst) => {
            format!(
                "\tmov{} {}, {}\n",
                suffix(t),
                show_operand(t, src),
                show_operand(t, dst)
            )
        }
        assembly::Instruction::Unary(operator, t, dst) => {
            format!(
                "\t{}{} {}\n",
                show_unary_instruction(operator),
                suffix(t),
                show_operand(t, dst)
            )
        }
        assembly::Instruction::Binary { op, t, src, dst } => {
            format!(
                "\t{}{} {}, {}\n",
                show_binary_instruction(op),
                suffix(t),
                show_operand(t, src),
                show_operand(t, dst)
            )
        }
        assembly::Instruction::Cmp(t, src, dst) => {
            format!(
                "\tcmp{} {}, {}\n",
                suffix(t),
                show_operand(t, src),
                show_operand(t, dst)
            )
        }
        assembly::Instruction::Idiv(t, operand) => {
            format!("\tidiv{} {}\n", suffix(t), show_operand(t, operand))
        }
        assembly::Instruction::Cdq(assembly::AsmType::Longword) => "\tcdq\n".to_string(),
        assembly::Instruction::Cdq(assembly::AsmType::Quadword) => "\tcdo\n".to_string(),
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
            format!(
                "\tpushq {}\n",
                show_operand(assembly::AsmType::Quadword, op)
            )
        }
        assembly::Instruction::Call(f) => {
            format!("\tcall {}\n", show_fun_name(f))
        }
        assembly::Instruction::Movsx(src, dst) => {
            format!(
                "\tmovslq {}, {}\n",
                show_operand(assembly::AsmType::Longword, src),
                show_operand(assembly::AsmType::Quadword, dst)
            )
        }
        assembly::Instruction::Ret => "
\tmovq %rbp, %rsp
\tpopq %rbp
\tret
"
        .to_string(),
    }
}

fn emit_global_directive(global: bool, label: String) -> String {
    if global {
        format!("\t.globl {}\n", label)
    } else {
        "".to_string()
    }
}

fn emit_zero_init(ini: initializers::StaticInit) -> String {
    match ini {
        initializers::StaticInit::IntInit(_) => "\t.zero 4\n".to_string(),
        initializers::StaticInit::LongInit(_) => "\t.zero 8\n".to_string(),
    }
}

fn emit_init(ini: initializers::StaticInit) -> String {
    match ini {
        initializers::StaticInit::IntInit(i) => format!("\t.quad {}\n", i),
        initializers::StaticInit::LongInit(l) => format!("\t.quad {}\n", l),
    }
}

fn emit_tl(f: assembly::TopLevel) -> String {
    match f {
        assembly::TopLevel::Function {
            name,
            global,
            instructions,
        } => {
            let label = show_label(name);
            let mut result = String::new();
            result.push_str(&emit_global_directive(global, label.clone()));
            result.push_str(
                format!(
                    "
\t.text
{}:
\tpushq %rbp
\tmovq %rsp, %rbp
",
                    label
                )
                .as_str(),
            );
            for instruction in instructions {
                result.push_str(&emit_instruction(instruction));
            }
            result
        }
        assembly::TopLevel::StaticVariable {
            name,
            alignment,
            global,
            init,
        } if initializers::is_zero(init) => {
            let mut result = String::new();
            let label = show_label(name);
            result.push_str(&emit_global_directive(global, label.clone()));
            result.push_str(
                format!(
                    "
\t.bss
\t{} {}
{}:
{}
",
                    align_directive(),
                    alignment,
                    label,
                    emit_zero_init(init),
                )
                .as_str(),
            );
            result
        }
        assembly::TopLevel::StaticVariable {
            name,
            alignment,
            global,
            init,
        } => {
            let mut result = String::new();
            let label = show_label(name);
            result.push_str(&emit_global_directive(global, label.clone()));
            result.push_str(
                format!(
                    "
\t.data
\t{} {}
{}:
{}",
                    align_directive(),
                    alignment,
                    label,
                    emit_init(init),
                )
                .as_str(),
            );
            result
        }
    }
}

fn emit_stack_note() -> String {
    "\t.section .note.GNU-stack,\"\",@progbits\n".to_string()
}

pub fn emit(program: assembly::T) {
    match program {
        assembly::T::Program(tls) => {
            for tl in tls {
                print!("{}", emit_tl(tl));
            }
            println!();
            print!("{}", emit_stack_note());
        }
    }
}
