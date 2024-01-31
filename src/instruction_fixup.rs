use crate::{assembly, rounding, symbols};

fn fixup_instruction(instruction: assembly::Instruction) -> Vec<assembly::Instruction> {
    match instruction {
        // mov指令不能将一个值从一个内存地址移动到另一个内存地址
        assembly::Instruction::Mov(
            src @ (assembly::Operand::Stack(_) | assembly::Operand::Data(_)),
            dst @ (assembly::Operand::Stack(_) | assembly::Operand::Data(_)),
        ) => vec![
            assembly::Instruction::Mov(src, assembly::Operand::Reg(assembly::Reg::R10)),
            assembly::Instruction::Mov(assembly::Operand::Reg(assembly::Reg::R10), dst),
        ],
        // idiv指令不能以常量作为操作数
        assembly::Instruction::Idiv(assembly::Operand::Imm(i)) => vec![
            assembly::Instruction::Mov(
                assembly::Operand::Imm(i),
                assembly::Operand::Reg(assembly::Reg::R10),
            ),
            assembly::Instruction::Idiv(assembly::Operand::Reg(assembly::Reg::R10)),
        ],
        assembly::Instruction::Binary {
            op: op @ (assembly::BinaryOperator::Add | assembly::BinaryOperator::Sub),
            src: src @ (assembly::Operand::Stack(_) | assembly::Operand::Data(_)),
            dst: dst @ (assembly::Operand::Stack(_) | assembly::Operand::Data(_)),
        } => vec![
            assembly::Instruction::Mov(src, assembly::Operand::Reg(assembly::Reg::R10)),
            assembly::Instruction::Binary {
                op: op,
                src: assembly::Operand::Reg(assembly::Reg::R10),
                dst: dst,
            },
        ],
        assembly::Instruction::Binary {
            op: assembly::BinaryOperator::Mult,
            src,
            dst: dst @ (assembly::Operand::Stack(_) | assembly::Operand::Data(_)),
        } => vec![
            assembly::Instruction::Mov(dst.clone(), assembly::Operand::Reg(assembly::Reg::R11)),
            assembly::Instruction::Binary {
                op: assembly::BinaryOperator::Mult,
                src,
                dst: assembly::Operand::Reg(assembly::Reg::R11),
            },
            assembly::Instruction::Mov(assembly::Operand::Reg(assembly::Reg::R11), dst),
        ],
        assembly::Instruction::Cmp(
            src @ (assembly::Operand::Stack(_) | assembly::Operand::Data(_)),
            dst @ (assembly::Operand::Stack(_) | assembly::Operand::Data(_)),
        ) => vec![
            assembly::Instruction::Mov(src.clone(), assembly::Operand::Reg(assembly::Reg::R10)),
            assembly::Instruction::Cmp(assembly::Operand::Reg(assembly::Reg::R10), dst),
        ],
        assembly::Instruction::Cmp(src, assembly::Operand::Imm(i)) => vec![
            assembly::Instruction::Mov(
                assembly::Operand::Imm(i),
                assembly::Operand::Reg(assembly::Reg::R11),
            ),
            assembly::Instruction::Cmp(src, assembly::Operand::Reg(assembly::Reg::R11)),
        ],
        other => vec![other],
    }
}

fn fixup_tl(f: assembly::TopLevel) -> assembly::TopLevel {
    match f {
        assembly::TopLevel::Function {
            name,
            global,
            instructions,
        } => {
            let stack_bytes = -symbols::get_bytes_required(name.clone());
            let mut _instructions = vec![assembly::Instruction::AllocateStack(
                rounding::round_way_from_zero(16, stack_bytes),
            )];
            for i in instructions {
                _instructions.append(&mut fixup_instruction(i));
            }
            assembly::TopLevel::Function {
                name: name,
                global: global,
                instructions: _instructions,
            }
        }
        static_var => static_var,
    }
}

pub fn fixup_program(program: assembly::T) -> assembly::T {
    match program {
        assembly::T::Program(tls) => {
            let mut fixed_functions = vec![];
            for tl in tls {
                fixed_functions.push(fixup_tl(tl));
            }
            assembly::T::Program(fixed_functions)
        }
    }
}
