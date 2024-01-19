use crate::{ast, ir, lexer, parser, unique_ids};

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

fn emit_ir_for_exp(exp: ast::Exp) -> (Vec<ir::Instruction>, ir::IrValue) {
    match exp {
        ast::Exp::Constant(c) => (vec![], ir::IrValue::Constant(c)),
        ast::Exp::Var(v) => (vec![], ir::IrValue::Var(v)),
        ast::Exp::Unary(op, inner) => emit_unary_expression(op, inner),
        ast::Exp::Binary(ast::BinaryOperator::And, e1, e2) => emit_and_expression(e1, e2),
        ast::Exp::Binary(ast::BinaryOperator::Or, e1, e2) => emit_or_expression(e1, e2),
        ast::Exp::Binary(op, e1, e2) => emit_binary_expression(op, e1, e2),
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
        } => emit_conditional_expression(condition, then_result, else_result),
    }
}

fn emit_unary_expression(
    op: ast::UnaryOperator,
    inner: Box<ast::Exp>,
) -> (Vec<ir::Instruction>, ir::IrValue) {
    let (mut eval_inner, v) = emit_ir_for_exp(*inner);
    let dst_name = unique_ids::make_temporary();
    let dst = ir::IrValue::Var(dst_name);
    let ir_op = convert_op(op);
    eval_inner.push(ir::Instruction::Unary {
        op: ir_op,
        src: v,
        dst: dst.clone(),
    });
    (eval_inner, dst)
}

fn emit_binary_expression(
    op: ast::BinaryOperator,
    e1: Box<ast::Exp>,
    e2: Box<ast::Exp>,
) -> (Vec<ir::Instruction>, ir::IrValue) {
    let (mut eval_v1, v1) = emit_ir_for_exp(*e1);
    let (mut eval_v2, v2) = emit_ir_for_exp(*e2);
    let dst_name = unique_ids::make_temporary();
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
    instructions.push(ir::Instruction::Copy { src: v1, dst: dst.clone() });
    instructions.push(ir::Instruction::Jump(end_label.clone()));
    instructions.push(ir::Instruction::Label(else_label));
    instructions.append(&mut eval_v2);
    instructions.push(ir::Instruction::Copy { src: v2, dst: dst.clone() });
    instructions.push(ir::Instruction::Label(end_label));
    (instructions, dst)
}

fn emit_ir_for_statement(statement: ast::Statement) -> Vec<ir::Instruction> {
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
        ast::Statement::If { condition, then_clause, else_clause } => {
            emit_ir_for_if_statement(condition, then_clause, else_clause)
        }
        ast::Statement::Null => vec![],
    }
}

fn emit_ir_for_block_item(declaration: ast::BlockItem) -> Vec<ir::Instruction> {
    match declaration {
        ast::BlockItem::S(s) => emit_ir_for_statement(s),
        ast::BlockItem::D(ast::Declaration {
            name,
            init: Some(e),
        }) => {
            let (eval_assignment, _assign_result) = emit_ir_for_exp(ast::Exp::Assignment(
                Box::new(ast::Exp::Var(name)),
                Box::new(e),
            ));
            eval_assignment
        }
        ast::BlockItem::D(ast::Declaration {
            name: _,
            init: None,
        }) => vec![],
    }
}

fn emit_ir_for_if_statement(condition: ast::Exp, then_clause: Box<ast::Statement>, else_clause: Option<Box<ast::Statement>>) -> Vec<ir::Instruction> {
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

fn emit_ir_for_function(f: ast::FunctionDefinition) -> ir::FunctionDefinition {
    match f {
        ast::FunctionDefinition::Function { name, body } => {
            let mut body_instructions = vec![];
            for b in body {
                body_instructions.append(&mut emit_ir_for_block_item(b));
            }
            let extra_return = ir::Instruction::Return(ir::IrValue::Constant(0));
            body_instructions.push(extra_return);
            ir::FunctionDefinition::Function {
                name: name,
                body: body_instructions,
            }
        }
    }
}

pub fn gen(fn_def: ast::Program) -> ir::Program {
    match fn_def {
        ast::Program::FunctionDefinition(f) => {
            ir::Program::FunctionDefinition(emit_ir_for_function(f))
        }
    }
}

#[test]
fn test_1() {
    let prog = "
    int main(void) {
        return 1 + 2 * 3;
    }
    ";
    let mut lexer = lexer::Lexer::new(prog.as_bytes());
    let tokens = lexer.lex();
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse();
    println!("{:?}", ast);
    let ir = gen(ast);
    println!("{}", ir);
}

#[test]
fn test_2() {
    let prog = "
    int main(void) {
        return 0 <= 2;
    }
    ";
    let mut lexer = lexer::Lexer::new(prog.as_bytes());
    let tokens = lexer.lex();
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse();
    println!("{:?}", ast);
    let ir = gen(ast);
    println!("{}", ir);
}

#[test]
fn test_3() {
    let prog = "
    int main(void) {
        return 1 || 0;
    }
    ";
    let mut lexer = lexer::Lexer::new(prog.as_bytes());
    let tokens = lexer.lex();
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse();
    println!("{:?}", ast);
    let ir = gen(ast);
    println!("{}", ir);
}

#[test]
fn test_4() {
    let prog = "
    int main(void) {
        return 2 == 2 || 0;
    }
    ";
    let mut lexer = lexer::Lexer::new(prog.as_bytes());
    let tokens = lexer.lex();
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse();
    println!("{:?}", ast);
    let ir = gen(ast);
    println!("{}", ir);
}
