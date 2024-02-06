use crate::{
    ast,
    ir::{self, IrValue},
    symbols, type_utils, types, unique_ids,
};

fn break_label(id: String) -> String {
    format!("break.{}", id)
}

fn continue_label(id: String) -> String {
    format!("continue.{}", id)
}

fn create_tmp(t: types::Type) -> String {
    let name = unique_ids::make_temporary();
    symbols::add_automatic_var(name, t);
    name
}

fn convert_op(op: ast::UnaryOperator) -> ir::UnaryOperator {
    match op {
        ast::UnaryOperator::Complement => ir::UnaryOperator::Complement,
        ast::UnaryOperator::Negate => ir::UnaryOperator::Negate,
        ast::UnaryOperator::Not => ir::UnaryOperator::Not,
    }
}

fn convert_binop(op: ast::BinaryOperator) -> ir::BinaryOperator {
    match op {
        ast::BinaryOperator::Add => ir::BinaryOperator::Add,
        ast::BinaryOperator::Subtract => ir::BinaryOperator::Subtract,
        ast::BinaryOperator::Multiply => ir::BinaryOperator::Multiply,
        ast::BinaryOperator::Divide => ir::BinaryOperator::Divide,
        ast::BinaryOperator::Mod => ir::BinaryOperator::Mod,
        ast::BinaryOperator::Equal => ir::BinaryOperator::Equal,
        ast::BinaryOperator::NotEqual => ir::BinaryOperator::NotEqual,
        ast::BinaryOperator::LessThan => ir::BinaryOperator::LessThan,
        ast::BinaryOperator::LessOrEqual => ir::BinaryOperator::LessOrEqual,
        ast::BinaryOperator::GreaterThan => ir::BinaryOperator::GreaterThan,
        ast::BinaryOperator::GreaterOrEqual => ir::BinaryOperator::GreaterOrEqual,
        ast::BinaryOperator::And | ast::BinaryOperator::Or => panic!("无法转换成ir运算符"),
    }
}

fn emit_ir_for_exp(exp: ast::TypedExp) -> (Vec<ir::Instruction>, ir::IrValue) {
    match exp.e {
        ast::Exp::Constant(c) => (vec![], ir::IrValue::Constant(c)),
        ast::Exp::Var(v) => (vec![], ir::IrValue::Var(v)),
        ast::Exp::Cast { target_type, e } => emit_cast_expression(target_type, e),
        ast::Exp::Unary(op, inner) => emit_unary_expression(exp.t, op, inner),
        ast::Exp::Binary(ast::BinaryOperator::And, e1, e2) => emit_and_expression(e1, e2),
        ast::Exp::Binary(ast::BinaryOperator::Or, e1, e2) => emit_or_expression(e1, e2),
        ast::Exp::Binary(op, e1, e2) => emit_binary_expression(exp.t, op, e1, e2),
        ast::Exp::Assignment(lhs, rhs) => match *lhs {
            ast::Exp::Var(v) => {
                let (mut rhs_instructions, rhs_result) = emit_ir_for_exp(*rhs);
                rhs_instructions.push(ir::Instruction::Copy {
                    src: rhs_result,
                    dst: ir::IrValue::Var(v.clone()),
                });
                (rhs_instructions, ir::IrValue::Var(v))
            }
            _ => panic!("错误的左值。"),
        },
        ast::Exp::Conditional {
            condition,
            then_result,
            else_result,
        } => emit_conditional_expression(exp.t, condition, then_result, else_result),
        ast::Exp::FunCall { f, args } => emit_fun_call(f, args),
    }
}

fn emit_unary_expression(
    t: types::Type,
    op: ast::UnaryOperator,
    inner: Box<ast::Exp>,
) -> (Vec<ir::Instruction>, ir::IrValue) {
    let (mut eval_inner, v) = emit_ir_for_exp(*inner);
    let dst_name = create_tmp(t);
    let dst = ir::IrValue::Var(dst_name);
    let ir_op = convert_op(op);
    eval_inner.push(ir::Instruction::Unary {
        op: ir_op,
        src: v,
        dst: dst.clone(),
    });
    (eval_inner, dst)
}

fn emit_cast_expression(
    target_type: types::Type,
    inner: ast::Exp,
) -> (Vec<ir::Instruction>, ir::IrValue) {
    let (mut eval_inner, result) = emit_ir_for_exp(inner);
    if type_utils::get_type(inner) == target_type {
        (eval_inner, result)
    } else {
        let dst_name = create_tmp(target_type);
        let dst = ir::IrValue::Var(dst_name);
        let cast_instruction = match target_type {
            types::Type::Long => ir::Instruction::SignExtend {
                src: result,
                dst: dst,
            },
            types::Type::Int => ir::Instruction::Truncate {
                src: result,
                dst: dst,
            },
            _ => panic!("内部错误：cast to function type."),
        };
        let mut instructions = vec![];
        instructions.append(&mut eval_inner);
        instructions.push(cast_instruction);
        (instructions, dst)
    }
}

fn emit_binary_expression(
    t: types::Type,
    op: ast::BinaryOperator,
    e1: Box<ast::Exp>,
    e2: Box<ast::Exp>,
) -> (Vec<ir::Instruction>, ir::IrValue) {
    let (mut eval_v1, v1) = emit_ir_for_exp(*e1);
    let (mut eval_v2, v2) = emit_ir_for_exp(*e2);
    let dst_name = create_tmp(t);
    let dst = ir::IrValue::Var(dst_name);
    let ir_op = convert_binop(op);
    let mut instructions = vec![];
    instructions.append(&mut eval_v1);
    instructions.append(&mut eval_v2);
    instructions.push(ir::Instruction::Binary {
        op: ir_op,
        src1: v1,
        src2: v2,
        dst: dst.clone(),
    });
    (instructions, dst)
}

fn emit_and_expression(
    e1: Box<ast::Exp>,
    e2: Box<ast::Exp>,
) -> (Vec<ir::Instruction>, ir::IrValue) {
    let (mut eval_v1, v1) = emit_ir_for_exp(*e1);
    let (mut eval_v2, v2) = emit_ir_for_exp(*e2);
    let false_label = unique_ids::make_label("and_false".to_string());
    let end_label = unique_ids::make_label("and_end".to_string());
    let dst_name = unique_ids::make_temporary();
    let dst = ir::IrValue::Var(dst_name);
    let mut instructions = vec![];
    instructions.append(&mut eval_v1);
    instructions.push(ir::Instruction::JumpIfZero(v1, false_label.clone()));
    instructions.append(&mut eval_v2);
    instructions.push(ir::Instruction::JumpIfZero(v2, false_label.clone()));
    instructions.push(ir::Instruction::Copy {
        src: ir::IrValue::Constant(1),
        dst: dst.clone(),
    });
    instructions.push(ir::Instruction::Jump(end_label.clone()));
    instructions.push(ir::Instruction::Label(false_label));
    instructions.push(ir::Instruction::Copy {
        src: ir::IrValue::Constant(0),
        dst: dst.clone(),
    });
    instructions.push(ir::Instruction::Label(end_label));
    (instructions, dst)
}

fn emit_or_expression(e1: Box<ast::Exp>, e2: Box<ast::Exp>) -> (Vec<ir::Instruction>, ir::IrValue) {
    let (mut eval_v1, v1) = emit_ir_for_exp(*e1);
    let (mut eval_v2, v2) = emit_ir_for_exp(*e2);
    let true_label = unique_ids::make_label("or_true".to_string());
    let end_label = unique_ids::make_label("or_end".to_string());
    let dst_name = unique_ids::make_temporary();
    let dst = ir::IrValue::Var(dst_name);
    let mut instructions = vec![];
    instructions.append(&mut eval_v1);
    instructions.push(ir::Instruction::JumpIfNotZero(v1, true_label.clone()));
    instructions.append(&mut eval_v2);
    instructions.push(ir::Instruction::JumpIfNotZero(v2, true_label.clone()));
    instructions.push(ir::Instruction::Copy {
        src: ir::IrValue::Constant(0),
        dst: dst.clone(),
    });
    instructions.push(ir::Instruction::Jump(end_label.clone()));
    instructions.push(ir::Instruction::Label(true_label));
    instructions.push(ir::Instruction::Copy {
        src: ir::IrValue::Constant(1),
        dst: dst.clone(),
    });
    instructions.push(ir::Instruction::Label(end_label));
    (instructions, dst)
}

fn emit_conditional_expression(
    t: types::Type,
    condition: Box<ast::Exp>,
    then_result: Box<ast::Exp>,
    else_result: Box<ast::Exp>,
) -> (Vec<ir::Instruction>, ir::IrValue) {
    let (mut eval_cond, c) = emit_ir_for_exp(*condition);
    let (mut eval_v1, v1) = emit_ir_for_exp(*then_result);
    let (mut eval_v2, v2) = emit_ir_for_exp(*else_result);
    let else_label = unique_ids::make_label("conditional_else".to_string());
    let end_label = unique_ids::make_label("conditional_end".to_string());
    let dst_name = unique_ids::make_temporary();
    let dst = ir::IrValue::Var(dst_name);
    let mut instructions = vec![];
    instructions.append(&mut eval_cond);
    instructions.push(ir::Instruction::JumpIfZero(c, else_label.clone()));
    instructions.append(&mut eval_v1);
    instructions.push(ir::Instruction::Copy {
        src: v1,
        dst: dst.clone(),
    });
    instructions.push(ir::Instruction::Jump(end_label.clone()));
    instructions.push(ir::Instruction::Label(else_label));
    instructions.append(&mut eval_v2);
    instructions.push(ir::Instruction::Copy {
        src: v2,
        dst: dst.clone(),
    });
    instructions.push(ir::Instruction::Label(end_label));
    (instructions, dst)
}

fn emit_ir_for_statement(statement: ast::Statement<ast::TypedExp>) -> Vec<ir::Instruction> {
    match statement {
        ast::Statement::Return(e) => {
            let (mut eval_exp, v) = emit_ir_for_exp(e);
            eval_exp.push(ir::Instruction::Return(v));
            eval_exp
        }
        ast::Statement::Expression(e) => {
            let (eval_exp, _exp_result) = emit_ir_for_exp(e);
            eval_exp
        }
        ast::Statement::If {
            condition,
            then_clause,
            else_clause,
        } => emit_ir_for_if_statement(condition, then_clause, else_clause),
        ast::Statement::Compound(ast::Block::Block(items)) => {
            let mut instructions = vec![];
            for item in items {
                instructions.append(&mut emit_ir_for_block_item(item));
            }
            instructions
        }
        ast::Statement::Break(id) => vec![ir::Instruction::Jump(break_label(id))],
        ast::Statement::Continue(id) => vec![ir::Instruction::Jump(continue_label(id))],
        ast::Statement::DoWhile {
            body,
            condition,
            id,
        } => emit_ir_for_do_loop(body, condition, id),
        ast::Statement::While {
            condition,
            body,
            id,
        } => emit_ir_for_while_loop(condition, body, id),
        ast::Statement::For {
            init,
            condition,
            post,
            body,
            id,
        } => emit_ir_for_for_loop(init, condition, post, body, id),
        ast::Statement::Null => vec![],
    }
}

fn emit_ir_for_block_item(declaration: ast::BlockItem<ast::TypedExp>) -> Vec<ir::Instruction> {
    match declaration {
        ast::BlockItem::S(s) => emit_ir_for_statement(s),
        ast::BlockItem::D(d) => emit_local_declaration(d),
    }
}

fn emit_local_declaration(d: ast::Declaration<ast::TypedExp>) -> Vec<ir::Instruction> {
    match d {
        ast::Declaration::VarDecl(vd) => emit_var_declaration(vd),
        ast::Declaration::FunDecl(_) => vec![],
    }
}

fn emit_var_declaration(vd: ast::VariableDeclaration<ast::TypedExp>) -> Vec<ir::Instruction> {
    match vd {
        ast::VariableDeclaration {
            name,
            var_type: _,
            init: Some(e),
            storage_class: _,
        } => {
            let (eval_assignment, _) = emit_ir_for_exp(ast::Exp::Assignment(
                Box::new(ast::Exp::Var(name)),
                Box::new(e),
            ));
            eval_assignment
        }
        ast::VariableDeclaration {
            name: _,
            var_type: _,
            init: None,
            storage_class: _,
        } => vec![],
    }
}

fn emit_ir_for_if_statement(
    condition: ast::Exp,
    then_clause: Box<ast::Statement<ast::TypedExp>>,
    else_clause: Option<Box<ast::Statement<ast::TypedExp>>>,
) -> Vec<ir::Instruction> {
    match else_clause {
        None => {
            let end_label = unique_ids::make_label("if_end".to_string());
            let (mut eval_condition, c) = emit_ir_for_exp(condition);
            let mut instructions = vec![];
            instructions.append(&mut eval_condition);
            instructions.push(ir::Instruction::JumpIfZero(c, end_label.clone()));
            instructions.append(&mut emit_ir_for_statement(*then_clause));
            instructions.push(ir::Instruction::Label(end_label));
            instructions
        }
        Some(_else_clause) => {
            let else_label = unique_ids::make_label("else".to_string());
            let end_label = unique_ids::make_label("".to_string());
            let (mut eval_condition, c) = emit_ir_for_exp(condition);
            let mut instructions = vec![];
            instructions.append(&mut eval_condition);
            instructions.push(ir::Instruction::JumpIfZero(c, else_label.clone()));
            instructions.append(&mut emit_ir_for_statement(*then_clause));
            instructions.push(ir::Instruction::Jump(end_label.clone()));
            instructions.push(ir::Instruction::Label(else_label));
            instructions.append(&mut emit_ir_for_statement(*_else_clause));
            instructions.push(ir::Instruction::Label(end_label));
            instructions
        }
    }
}

fn emit_ir_for_do_loop(
    body: Box<ast::Statement<ast::TypedExp>>,
    condition: ast::Exp,
    id: String,
) -> Vec<ir::Instruction> {
    let start_label = unique_ids::make_label("do_loop_start".to_string());
    let cont_label = continue_label(id.clone());
    let br_label = break_label(id);
    let (mut eval_condition, c) = emit_ir_for_exp(condition);
    let mut instructions = vec![];
    instructions.push(ir::Instruction::Label(start_label.clone()));
    instructions.append(&mut emit_ir_for_statement(*body));
    instructions.push(ir::Instruction::Label(cont_label));
    instructions.append(&mut eval_condition);
    instructions.push(ir::Instruction::JumpIfNotZero(c, start_label));
    instructions.push(ir::Instruction::Label(br_label));
    instructions
}

fn emit_ir_for_while_loop(
    condition: ast::Exp,
    body: Box<ast::Statement<ast::TypedExp>>,
    id: String,
) -> Vec<ir::Instruction> {
    let cont_label = continue_label(id.clone());
    let br_label = break_label(id);
    let (mut eval_condition, c) = emit_ir_for_exp(condition);
    let mut instructions = vec![];
    instructions.push(ir::Instruction::Label(cont_label.clone()));
    instructions.append(&mut eval_condition);
    instructions.push(ir::Instruction::JumpIfZero(c, br_label.clone()));
    instructions.append(&mut emit_ir_for_statement(*body));
    instructions.push(ir::Instruction::Jump(cont_label));
    instructions.push(ir::Instruction::Label(br_label));
    instructions
}

fn emit_ir_for_for_loop(
    init: ast::ForInit<ast::TypedExp>,
    condition: Option<ast::Exp>,
    post: Option<ast::Exp>,
    body: Box<ast::Statement<ast::TypedExp>>,
    id: String,
) -> Vec<ir::Instruction> {
    let start_label = unique_ids::make_label("for_start".to_string());
    let cont_label = continue_label(id.clone());
    let br_label = break_label(id);
    let mut for_init_instructions = match init {
        ast::ForInit::InitDecl(d) => emit_var_declaration(d),
        ast::ForInit::InitExp(e) => match e {
            Some(_e) => {
                let (instrs, _) = emit_ir_for_exp(_e);
                instrs
            }
            None => vec![],
        },
    };
    let mut test_condition = match condition {
        Some(_condition) => {
            let (mut instrs, v) = emit_ir_for_exp(_condition);
            instrs.push(ir::Instruction::JumpIfZero(v, br_label.clone()));
            instrs
        }
        None => vec![],
    };
    let mut post_instructions = match post {
        Some(_post) => {
            let (instrs, _post_result) = emit_ir_for_exp(_post);
            instrs
        }
        None => vec![],
    };
    for_init_instructions.push(ir::Instruction::Label(start_label.clone()));
    for_init_instructions.append(&mut test_condition);
    for_init_instructions.append(&mut emit_ir_for_statement(*body));
    for_init_instructions.push(ir::Instruction::Label(cont_label));
    for_init_instructions.append(&mut post_instructions);
    for_init_instructions.push(ir::Instruction::Jump(start_label));
    for_init_instructions.push(ir::Instruction::Label(br_label));
    for_init_instructions
}

fn emit_fun_call(f: String, args: Vec<ast::Exp>) -> (Vec<ir::Instruction>, IrValue) {
    let dst_name = unique_ids::make_temporary();
    let dst = ir::IrValue::Var(dst_name);
    let mut arg_instructions = vec![];
    let mut arg_vals = vec![];
    for arg in args {
        let mut t = emit_ir_for_exp(arg);
        arg_instructions.append(&mut t.0);
        arg_vals.push(t.1);
    }
    arg_instructions.push(ir::Instruction::FunCall {
        f: f,
        args: arg_vals,
        dst: dst.clone(),
    });
    (arg_instructions, dst)
}

fn emit_fun_declaration(fd: ast::Declaration<ast::TypedExp>) -> Option<ir::TopLevel> {
    match fd {
        ast::Declaration::FunDecl(ast::FunctionDeclaration {
            name,
            fun_type: _,
            params,
            body: Some(ast::Block::Block(block_items)),
            storage_class: _,
        }) => {
            let global = symbols::is_global(name.clone());
            let mut body_instructions = vec![];
            for i in block_items {
                body_instructions.append(&mut emit_ir_for_block_item(i));
            }
            let extra_return = ir::Instruction::Return(ir::IrValue::Constant(0));
            body_instructions.push(extra_return);
            Some(ir::TopLevel::Function {
                name: name,
                global: global,
                params: params,
                body: body_instructions,
            })
        }
        _ => None,
    }
}

fn convert_symbols_to_ir(all_symbols: Vec<(String, symbols::Entry)>) -> Vec<ir::TopLevel> {
    let mut arr = vec![];
    for (name, entry) in all_symbols {
        let symbol_ir = match entry.attrs {
            symbols::IdentifierAttrs::StaticAttr { init, global } => match init {
                symbols::InitialValue::Initial(i) => Some(ir::TopLevel::StaticVariable {
                    name: name,
                    t: entry.t,
                    global: global,
                    init: i,
                }),
                symbols::InitialValue::Tentative => Some(ir::TopLevel::StaticVariable {
                    name: name,
                    t: entry.t,
                    global: global,
                    init: 0,
                }),
                symbols::InitialValue::NoInitializer => None,
            },
            _ => None,
        };
        if let Some(_symbol_ir) = symbol_ir {
            arr.push(_symbol_ir);
        }
    }
    arr
}

pub fn gen(program: ast::TypedProgType) -> ir::T {
    match program {
        ast::TypedProgType::Program(decls) => {
            let mut ir_fn_defs = vec![];
            for fn_def in decls {
                if let Some(fn_def_ir) = emit_fun_declaration(fn_def) {
                    ir_fn_defs.push(fn_def_ir);
                }
            }
            let mut ir_var_defs = convert_symbols_to_ir(symbols::bindings());
            let mut result = vec![];
            result.append(&mut ir_fn_defs);
            result.append(&mut ir_var_defs);
            ir::T::Program(result)
        }
    }
}
