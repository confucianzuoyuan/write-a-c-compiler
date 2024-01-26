use crate::{assembly, ir};

const PARAM_PASSING_REGS: [assembly::Reg; 6] = [
    assembly::Reg::DI,
    assembly::Reg::SI,
    assembly::Reg::DX,
    assembly::Reg::CX,
    assembly::Reg::R8,
    assembly::Reg::R9,
];

fn convert_val(ir_value: ir::IrValue) -> assembly::Operand {
    match ir_value {
        ir::IrValue::Constant(i) => assembly::Operand::Imm(i),
        ir::IrValue::Var(v) => assembly::Operand::Pseudo(v),
    }
}

fn convert_unop(ir_unop: ir::UnaryOperator) -> assembly::UnaryOperator {
    match ir_unop {
        ir::UnaryOperator::Complement => assembly::UnaryOperator::Not,
        ir::UnaryOperator::Negate => assembly::UnaryOperator::Neg,
        ir::UnaryOperator::Not => panic!("无法直接将not ir指令转换成汇编。\r\n"),
    }
}

fn convert_binop(ir_binop: ir::BinaryOperator) -> assembly::BinaryOperator {
    match ir_binop {
        ir::BinaryOperator::Add => assembly::BinaryOperator::Add,
        ir::BinaryOperator::Subtract => assembly::BinaryOperator::Sub,
        ir::BinaryOperator::Multiply => assembly::BinaryOperator::Mult,
        ir::BinaryOperator::Divide
        | ir::BinaryOperator::Mod
        | ir::BinaryOperator::Equal
        | ir::BinaryOperator::NotEqual
        | ir::BinaryOperator::GreaterOrEqual
        | ir::BinaryOperator::LessOrEqual
        | ir::BinaryOperator::GreaterThan
        | ir::BinaryOperator::LessThan => panic!("不是二元运算符。"),
    }
}

fn convert_cond_code(ir_cond_code: ir::BinaryOperator) -> assembly::CondCode {
    match ir_cond_code {
        ir::BinaryOperator::Equal => assembly::CondCode::E,
        ir::BinaryOperator::NotEqual => assembly::CondCode::NE,
        ir::BinaryOperator::GreaterThan => assembly::CondCode::G,
        ir::BinaryOperator::GreaterOrEqual => assembly::CondCode::GE,
        ir::BinaryOperator::LessThan => assembly::CondCode::L,
        ir::BinaryOperator::LessOrEqual => assembly::CondCode::LE,
        _ => panic!("不是条件码。"),
    }
}

fn convert_function_call(
    f: String,
    args: Vec<ir::IrValue>,
    dst: ir::IrValue,
) -> Vec<assembly::Instruction> {
    let mut reg_args = vec![];
    let mut stack_args = vec![];
    for (i, arg) in args.iter().enumerate() {
        if i <= 6 {
            reg_args.push(arg.clone());
        } else {
            stack_args.push(arg.clone());
        }
    }
    let stack_padding = if stack_args.len() % 2 == 0 { 0 } else { 8 };
    let mut instructions = if stack_padding == 0 {
        vec![]
    } else {
        vec![assembly::Instruction::AllocateStack(stack_padding)]
    };
    for (i, reg_arg) in reg_args.iter().enumerate() {
        let r = PARAM_PASSING_REGS[i].clone();
        let assembly_arg = convert_val(reg_arg.clone());
        instructions.push(assembly::Instruction::Mov(
            assembly_arg,
            assembly::Operand::Reg(r),
        ));
    }
    for (_, stack_arg) in stack_args.iter().rev().enumerate() {
        let assembly_arg = convert_val(stack_arg.clone());
        instructions.append(&mut match assembly_arg {
            assembly::Operand::Imm(_) | assembly::Operand::Reg(_) => {
                vec![assembly::Instruction::Push(assembly_arg)]
            }
            _ => vec![
                assembly::Instruction::Mov(assembly_arg, assembly::Operand::Reg(assembly::Reg::AX)),
                assembly::Instruction::Push(assembly::Operand::Reg(assembly::Reg::AX)),
            ],
        });
    }
    instructions.push(assembly::Instruction::Call(f));
    let bytes_to_remove = (8 * stack_args.len() as i64) + stack_padding;
    let mut dealloc = if bytes_to_remove == 0 {
        vec![]
    } else {
        vec![assembly::Instruction::DeallocateStack(bytes_to_remove)]
    };
    instructions.append(&mut dealloc);
    let assembly_dst = convert_val(dst);
    instructions.push(assembly::Instruction::Mov(
        assembly::Operand::Reg(assembly::Reg::AX),
        assembly_dst,
    ));
    instructions
}

fn convert_instruction(ir_instruction: ir::Instruction) -> Vec<assembly::Instruction> {
    match ir_instruction {
        ir::Instruction::Copy { src, dst } => {
            let asm_src = convert_val(src);
            let asm_dst = convert_val(dst);
            vec![assembly::Instruction::Mov(asm_src, asm_dst)]
        }
        ir::Instruction::Return(ir_value) => {
            let asm_val = convert_val(ir_value);
            vec![
                assembly::Instruction::Mov(asm_val, assembly::Operand::Reg(assembly::Reg::AX)),
                assembly::Instruction::Ret,
            ]
        }
        ir::Instruction::Unary {
            op: ir::UnaryOperator::Not,
            src,
            dst,
        } => {
            let asm_src = convert_val(src);
            let asm_dst = convert_val(dst);
            vec![
                assembly::Instruction::Cmp(assembly::Operand::Imm(0), asm_src),
                assembly::Instruction::Mov(assembly::Operand::Imm(0), asm_dst.clone()),
                assembly::Instruction::SetCC(assembly::CondCode::E, asm_dst),
            ]
        }
        ir::Instruction::Unary { op, src, dst } => {
            let asm_op = convert_unop(op);
            let asm_src = convert_val(src);
            let asm_dst = convert_val(dst);
            vec![
                assembly::Instruction::Mov(asm_src, asm_dst.clone()),
                assembly::Instruction::Unary(asm_op, asm_dst),
            ]
        }
        ir::Instruction::Binary {
            op,
            src1,
            src2,
            dst,
        } => {
            let asm_src1 = convert_val(src1);
            let asm_src2 = convert_val(src2);
            let asm_dst = convert_val(dst);
            match op {
                ir::BinaryOperator::Equal
                | ir::BinaryOperator::NotEqual
                | ir::BinaryOperator::GreaterThan
                | ir::BinaryOperator::GreaterOrEqual
                | ir::BinaryOperator::LessThan
                | ir::BinaryOperator::LessOrEqual => {
                    let cond_code = convert_cond_code(op);
                    vec![
                        assembly::Instruction::Cmp(asm_src2, asm_src1),
                        assembly::Instruction::Mov(assembly::Operand::Imm(0), asm_dst.clone()),
                        assembly::Instruction::SetCC(cond_code, asm_dst),
                    ]
                }
                ir::BinaryOperator::Divide | ir::BinaryOperator::Mod => {
                    let result_reg = match op {
                        ir::BinaryOperator::Divide => assembly::Reg::AX,
                        _ => assembly::Reg::DX,
                    };
                    vec![
                        assembly::Instruction::Mov(
                            asm_src1,
                            assembly::Operand::Reg(assembly::Reg::AX),
                        ),
                        assembly::Instruction::Cdq,
                        assembly::Instruction::Idiv(asm_src2),
                        assembly::Instruction::Mov(
                            assembly::Operand::Reg(result_reg),
                            asm_dst.clone(),
                        ),
                    ]
                }
                _ => {
                    let asm_op = convert_binop(op);
                    vec![
                        assembly::Instruction::Mov(asm_src1, asm_dst.clone()),
                        assembly::Instruction::Binary {
                            op: asm_op,
                            src: asm_src2,
                            dst: asm_dst,
                        },
                    ]
                }
            }
        }
        ir::Instruction::Jump(target) => vec![assembly::Instruction::Jmp(target)],
        ir::Instruction::JumpIfZero(cond, target) => {
            let asm_cond = convert_val(cond);
            vec![
                assembly::Instruction::Cmp(assembly::Operand::Imm(0), asm_cond),
                assembly::Instruction::JmpCC(assembly::CondCode::E, target),
            ]
        }
        ir::Instruction::JumpIfNotZero(cond, target) => {
            let asm_cond = convert_val(cond);
            vec![
                assembly::Instruction::Cmp(assembly::Operand::Imm(0), asm_cond),
                assembly::Instruction::JmpCC(assembly::CondCode::NE, target),
            ]
        }
        ir::Instruction::Label(l) => vec![assembly::Instruction::Label(l)],
        ir::Instruction::FunCall { f, args, dst } => convert_function_call(f, args, dst),
    }
}

fn pass_params(param_list: Vec<String>) -> Vec<assembly::Instruction> {
    let mut register_params = vec![];
    let mut stack_params = vec![];
    for (i, param) in param_list.iter().enumerate() {
        if i <= 6 {
            register_params.push(param.clone());
        } else {
            stack_params.push(param.clone());
        }
    }
    let mut instructions = vec![];
    for (i, param) in register_params.iter().enumerate() {
        let r = PARAM_PASSING_REGS[i].clone();
        instructions.push(assembly::Instruction::Mov(
            assembly::Operand::Reg(r),
            assembly::Operand::Pseudo(param.clone()),
        ));
    }
    for (i, param) in stack_params.iter().enumerate() {
        let stk = assembly::Operand::Stack(16 + (8 * i as i64));
        instructions.push(assembly::Instruction::Mov(
            stk,
            assembly::Operand::Pseudo(param.clone()),
        ))
    }
    instructions
}

fn convert_function(f: ir::FunctionDefinition) -> assembly::FunctionDefinition {
    match f {
        ir::FunctionDefinition::Function { name, params: _, body } => {
            let mut instructions = vec![];
            for instruction in body {
                instructions.append(&mut convert_instruction(instruction));
            }
            assembly::FunctionDefinition::Function {
                name: name,
                instructions: instructions,
            }
        }
    }
}

pub fn gen(program: ir::Program) -> assembly::Program {
    match program {
        ir::Program::FunctionDefinition(fn_defs) => {
            let mut arr = vec![];
            for fn_def in fn_defs {
                arr.push(convert_function(fn_def));
            }
            assembly::Program::FunctionDefinition(arr)
        }
    }
}
