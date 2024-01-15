use crate::{ast, ir, unique_ids, lexer, parser};

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
        ast::Exp::Unary(op, inner) => emit_unary_expression(op, inner),
        ast::Exp::Binary(ast::BinaryOperator::And, e1, e2) => emit_and_expression(e1, e2),
        ast::Exp::Binary(ast::BinaryOperator::Or, e1, e2) => emit_or_expression(e1, e2),
        ast::Exp::Binary(op, e1, e2) => emit_binary_expression(op, e1, e2),
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

fn emit_and_expression(e1: Box<ast::Exp>, e2: Box<ast::Exp>) -> (Vec<ir::Instruction>, ir::IrValue) {
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

fn emit_ir_for_statement(statement: ast::Statement) -> Vec<ir::Instruction> {
    match statement {
        ast::Statement::Return(e) => {
            let (mut eval_exp, v) = emit_ir_for_exp(e);
            eval_exp.push(ir::Instruction::Return(v));
            eval_exp
        }
    }
}

fn emit_ir_for_function(f: ast::FunctionDefinition) -> ir::FunctionDefinition {
    match f {
        ast::FunctionDefinition::Function { name, body } => {
            let instructions = emit_ir_for_statement(body);
            ir::FunctionDefinition::Function {
                name: name,
                body: instructions,
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