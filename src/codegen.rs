use crate::{assembly, assembly_symbols, constants, ir, symbols, type_utils, types};

const PARAM_PASSING_REGS: [assembly::Reg; 6] = [
    assembly::Reg::DI,
    assembly::Reg::SI,
    assembly::Reg::DX,
    assembly::Reg::CX,
    assembly::Reg::R8,
    assembly::Reg::R9,
];

const ZERO: assembly::Operand = assembly::Operand::Imm(0 as i64);

fn convert_val(ir_value: ir::IrValue) -> assembly::Operand {
    match ir_value {
        ir::IrValue::Constant(constants::T::ConstInt(i)) => assembly::Operand::Imm(i as i64),
        ir::IrValue::Constant(constants::T::ConstLong(i)) => assembly::Operand::Imm(i),
        ir::IrValue::Var(v) => assembly::Operand::Pseudo(v),
    }
}

fn convert_type(t: types::Type) -> assembly::AsmType {
    match t {
        types::Type::Int => assembly::AsmType::Longword,
        types::Type::Long => assembly::AsmType::Quadword,
        types::Type::FunType {
            param_types: _,
            ret_type: _,
        } => {
            panic!("内部错误，无法将函数类型转换成汇编代码。")
        }
    }
}

fn asm_type(t: ir::IrValue) -> assembly::AsmType {
    match t {
        ir::IrValue::Constant(constants::T::ConstLong(_)) => assembly::AsmType::Quadword,
        ir::IrValue::Constant(constants::T::ConstInt(_)) => assembly::AsmType::Longword,
        ir::IrValue::Var(v) => convert_type(symbols::get(v).t),
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
        if i < 6 {
            reg_args.push(arg.clone());
        } else {
            stack_args.push(arg.clone());
        }
    }
    let stack_padding = if stack_args.len() % 2 == 0 { 0 } else { 8 };
    let mut instructions = if stack_padding == 0 {
        vec![]
    } else {
        vec![assembly::Instruction::Binary {
            op: assembly::BinaryOperator::Sub,
            t: assembly::AsmType::Quadword,
            src: assembly::Operand::Imm(stack_padding as i64),
            dst: assembly::Operand::Reg(assembly::Reg::SP),
        }]
    };
    for (i, reg_arg) in reg_args.iter().enumerate() {
        let r = PARAM_PASSING_REGS[i].clone();
        let assembly_arg = convert_val(reg_arg.clone());
        instructions.push(assembly::Instruction::Mov(
            asm_type(reg_arg),
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
            _ => {
                let assemby_type = asm_type(stack_arg);
                if assemby_type == assembly::AsmType::Quadword {
                    vec![assembly::Instruction::Push(assembly::Operand::Reg(
                        assembly::Reg::AX,
                    ))]
                } else {
                    vec![
                        assembly::Instruction::Mov(
                            assemby_type,
                            assembly_arg,
                            assembly::Operand::Reg(assembly::Reg::AX),
                        ),
                        assembly::Instruction::Push(assembly::Operand::Reg(assembly::Reg::AX)),
                    ]
                }
            }
        });
    }
    instructions.push(assembly::Instruction::Call(f));
    let bytes_to_remove = (8 * stack_args.len() as i64) + stack_padding;
    let mut dealloc = if bytes_to_remove == 0 {
        vec![]
    } else {
        vec![assembly::Instruction::Binary {
            op: assembly::BinaryOperator::Add,
            t: assembly::AsmType::Quadword,
            src: assembly::Operand::Imm(bytes_to_remove as i64),
            dst: assembly::Operand::Reg(assembly::Reg::SP),
        }]
    };
    instructions.append(&mut dealloc);
    let assembly_dst = convert_val(dst);
    instructions.push(assembly::Instruction::Mov(
        asm_type(dst),
        assembly::Operand::Reg(assembly::Reg::AX),
        assembly_dst,
    ));
    instructions
}

fn convert_instruction(ir_instruction: ir::Instruction) -> Vec<assembly::Instruction> {
    match ir_instruction {
        ir::Instruction::Copy { src, dst } => {
            let t = asm_type(src);
            let asm_src = convert_val(src);
            let asm_dst = convert_val(dst);
            vec![assembly::Instruction::Mov(t, asm_src, asm_dst)]
        }
        ir::Instruction::Return(ir_value) => {
            let t = asm_type(ir_value);
            let asm_val = convert_val(ir_value);
            vec![
                assembly::Instruction::Mov(t, asm_val, assembly::Operand::Reg(assembly::Reg::AX)),
                assembly::Instruction::Ret,
            ]
        }
        ir::Instruction::Unary {
            op: ir::UnaryOperator::Not,
            src,
            dst,
        } => {
            let src_t = asm_type(src);
            let dst_t = asm_type(dst);
            let asm_src = convert_val(src);
            let asm_dst = convert_val(dst);
            vec![
                assembly::Instruction::Cmp(src_t, assembly::Operand::Imm(0), asm_src),
                assembly::Instruction::Mov(dst_t, assembly::Operand::Imm(0), asm_dst.clone()),
                assembly::Instruction::SetCC(assembly::CondCode::E, asm_dst),
            ]
        }
        ir::Instruction::Unary { op, src, dst } => {
            let t = asm_type(src);
            let asm_op = convert_unop(op);
            let asm_src = convert_val(src);
            let asm_dst = convert_val(dst);
            vec![
                assembly::Instruction::Mov(t, asm_src, asm_dst.clone()),
                assembly::Instruction::Unary(asm_op, t, asm_dst),
            ]
        }
        ir::Instruction::Binary {
            op,
            src1,
            src2,
            dst,
        } => {
            let src_t = asm_type(src1);
            let dst_t = asm_type(dst);
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
                        assembly::Instruction::Cmp(src_t, asm_src2, asm_src1),
                        assembly::Instruction::Mov(
                            dst_t,
                            assembly::Operand::Imm(0),
                            asm_dst.clone(),
                        ),
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
                            src_t,
                            asm_src1,
                            assembly::Operand::Reg(assembly::Reg::AX),
                        ),
                        assembly::Instruction::Cdq(src_t),
                        assembly::Instruction::Idiv(src_t, asm_src2),
                        assembly::Instruction::Mov(
                            src_t,
                            assembly::Operand::Reg(result_reg),
                            asm_dst.clone(),
                        ),
                    ]
                }
                _ => {
                    let asm_op = convert_binop(op);
                    vec![
                        assembly::Instruction::Mov(src_t, asm_src1, asm_dst.clone()),
                        assembly::Instruction::Binary {
                            op: asm_op,
                            t: src_t,
                            src: asm_src2,
                            dst: asm_dst,
                        },
                    ]
                }
            }
        }
        ir::Instruction::Jump(target) => vec![assembly::Instruction::Jmp(target)],
        ir::Instruction::JumpIfZero(cond, target) => {
            let t = asm_type(cond);
            let asm_cond = convert_val(cond);
            vec![
                assembly::Instruction::Cmp(t, assembly::Operand::Imm(0), asm_cond),
                assembly::Instruction::JmpCC(assembly::CondCode::E, target),
            ]
        }
        ir::Instruction::JumpIfNotZero(cond, target) => {
            let t = asm_type(cond);
            let asm_cond = convert_val(cond);
            vec![
                assembly::Instruction::Cmp(t, assembly::Operand::Imm(0), asm_cond),
                assembly::Instruction::JmpCC(assembly::CondCode::NE, target),
            ]
        }
        ir::Instruction::Label(l) => vec![assembly::Instruction::Label(l)],
        ir::Instruction::FunCall { f, args, dst } => convert_function_call(f, args, dst),
        ir::Instruction::SignExtend { src, dst } => {
            let asm_src = convert_val(src);
            let asm_dst = convert_val(dst);
            vec![assembly::Instruction::Movsx(asm_src, asm_dst)]
        }
        ir::Instruction::Truncate { src, dst } => {
            let asm_src = convert_val(src);
            let asm_dst = convert_val(dst);
            vec![assembly::Instruction::Mov(
                assembly::AsmType::Longword,
                asm_src,
                asm_dst,
            )]
        }
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
        let param_t = asm_type(ir::IrValue::Var(param));
        instructions.push(assembly::Instruction::Mov(
            param_t,
            assembly::Operand::Reg(r),
            assembly::Operand::Pseudo(param.clone()),
        ));
    }
    for (i, param) in stack_params.iter().enumerate() {
        let stk = assembly::Operand::Stack(16 + (8 * i as i64));
        let param_t = asm_type(ir::IrValue::Var(param));
        instructions.push(assembly::Instruction::Mov(
            param_t,
            stk,
            assembly::Operand::Pseudo(param.clone()),
        ))
    }
    instructions
}

fn convert_top_level(top_level: ir::TopLevel) -> assembly::TopLevel {
    match top_level {
        ir::TopLevel::Function {
            name,
            global,
            params,
            body,
        } => {
            let mut instructions = pass_params(params);
            for i in body {
                instructions.append(&mut convert_instruction(i));
            }
            assembly::TopLevel::Function {
                name: name,
                global: global,
                instructions: instructions,
            }
        }
        ir::TopLevel::StaticVariable {
            name,
            t,
            global,
            init,
        } => assembly::TopLevel::StaticVariable {
            name: name,
            alignment: type_utils::get_alignment(t),
            global: global,
            init: init,
        },
    }
}

fn convert_symbol(name: String, entry: symbols::Entry) {
    match entry {
        symbols::Entry {
            t: _,
            attrs:
                symbols::IdentifierAttrs::FunAttr {
                    defined,
                    global: _,
                    stack_frame_size: _,
                },
        } => assembly_symbols::add_fun(name, defined),
        symbols::Entry {
            t,
            attrs: symbols::IdentifierAttrs::StaticAttr { init: _, global: _ },
        } => assembly_symbols::add_var(name, convert_type(t), true),
        symbols::Entry { t, attrs: _ } => assembly_symbols::add_var(name, convert_type(t), false),
    }
}

pub fn gen(program: ir::T) -> assembly::T {
    match program {
        ir::T::Program(top_levels) => {
            let mut tls = vec![];
            for top_level in top_levels {
                tls.push(convert_top_level(top_level));
            }
            symbols::iter(convert_symbol);
            assembly::T::Program(tls)
        }
    }
}
