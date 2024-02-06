use std::process::id;

use crate::{ast, const_convert, constants, initializers, symbols, type_utils, types};

pub fn convert_to(e: Box<ast::Exp>, target_type: types::Type) -> ast::TypedExp {
    let cast = ast::Exp::Cast {
        target_type: target_type,
        e: e,
    };
    type_utils::set_type(cast, target_type)
}

pub fn get_comman_type(t1: types::Type, t2: types::Type) -> types::Type {
    if t1 == t2 {
        t1
    } else {
        types::Type::Long
    }
}

pub fn typecheck_var(v: String) -> ast::TypedExp {
    let v_type = symbols::get(v).t;
    let e = ast::Exp::Var(v);
    match v_type {
        types::Type::FunType {
            param_types: _,
            ret_type: _,
        } => panic!("试图将函数名用作变量。"),
        types::Type::Int | types::Type::Long => type_utils::set_type(e, v_type),
    }
}

pub fn typecheck_const(c: constants::T) -> ast::TypedExp {
    let e = ast::Exp::Constant(c);
    match c {
        constants::T::ConstInt(_) => type_utils::set_type(e, types::Type::Int),
        constants::T::ConstLong(_) => type_utils::set_type(e, types::Type::Long),
    }
}

pub fn typecheck_exp(exp: ast::Exp) -> ast::TypedExp {
    match exp {
        ast::Exp::FunCall { f, args } => typecheck_fun_call(f, args),
        ast::Exp::Var(v) => typecheck_var(v),
        ast::Exp::Cast {
            target_type,
            e: inner,
        } => {
            let cast_exp = ast::Exp::Cast {
                target_type: target_type,
                e: Box::new(typecheck_exp(*inner).e),
            };
            type_utils::set_type(cast_exp, target_type)
        }
        ast::Exp::Unary(op, inner) => typecheck_unary(op, *inner),
        ast::Exp::Binary(op, e1, e2) => typecheck_binary(op, *e1, *e2),
        ast::Exp::Assignment(lhs, rhs) => typecheck_assignment(*lhs, *rhs),
        ast::Exp::Conditional {
            condition,
            then_result,
            else_result,
        } => typecheck_conditional(*condition, *then_result, *else_result),
        ast::Exp::Constant(c) => typecheck_const(c),
    }
}

pub fn typecheck_unary(op: ast::UnaryOperator, inner: ast::Exp) -> ast::TypedExp {
    let typed_inner = typecheck_exp(inner);
    let unary_exp = ast::Exp::Unary(op, Box::new(typed_inner.e));
    match op {
        ast::UnaryOperator::Not => type_utils::set_type(unary_exp, types::Type::Int),
        _ => type_utils::set_type(unary_exp, type_utils::get_type(typed_inner)),
    }
}

pub fn typecheck_binary(op: ast::BinaryOperator, e1: ast::Exp, e2: ast::Exp) -> ast::TypedExp {
    let typed_e1 = typecheck_exp(e1);
    let typed_e2 = typecheck_exp(e2);
    match op {
        ast::BinaryOperator::Add | ast::BinaryOperator::Or => {
            let typed_binexp = ast::Exp::Binary(op, Box::new(typed_e1.e), Box::new(typed_e2.e));
            type_utils::set_type(typed_binexp, types::Type::Int)
        }
        _ => {
            let t1 = type_utils::get_type(typed_e1);
            let t2 = type_utils::get_type(typed_e2);
            let common_type = get_comman_type(t1, t2);
            let converted_e1 = convert_to(Box::new(typed_e1.e), common_type);
            let converted_e2 = convert_to(Box::new(typed_e2.e), common_type);
            let binary_exp =
                ast::Exp::Binary(op, Box::new(converted_e1.e), Box::new(converted_e2.e));
            match op {
                ast::BinaryOperator::Add
                | ast::BinaryOperator::Subtract
                | ast::BinaryOperator::Multiply
                | ast::BinaryOperator::Divide
                | ast::BinaryOperator::Mod => type_utils::set_type(binary_exp, common_type),
                _ => type_utils::set_type(binary_exp, types::Type::Int),
            }
        }
    }
}

pub fn typecheck_assignment(lhs: ast::Exp, rhs: ast::Exp) -> ast::TypedExp {
    let typed_lhs = typecheck_exp(lhs);
    let lhs_type = type_utils::get_type(typed_lhs);
    let typed_rhs = typecheck_exp(rhs);
    let converted_rhs = convert_to(Box::new(typed_rhs.e), lhs_type);
    let assign_exp = ast::Exp::Assignment(Box::new(typed_lhs.e), Box::new(converted_rhs.e));
    type_utils::set_type(assign_exp, lhs_type)
}

pub fn typecheck_conditional(
    condition: ast::Exp,
    then_exp: ast::Exp,
    else_exp: ast::Exp,
) -> ast::TypedExp {
    let typed_condition = typecheck_exp(condition);
    let typed_then = typecheck_exp(then_exp);
    let typed_else = typecheck_exp(else_exp);
    let common_type = get_comman_type(
        type_utils::get_type(typed_then),
        type_utils::get_type(typed_else),
    );
    let converted_then = convert_to(Box::new(typed_then.e), common_type);
    let converted_else = convert_to(Box::new(typed_else.e), common_type);
    let conditional_exp = ast::Exp::Conditional {
        condition: Box::new(typed_condition.e),
        then_result: Box::new(converted_then.e),
        else_result: Box::new(converted_else.e),
    };
    type_utils::set_type(conditional_exp, common_type)
}

pub fn typecheck_fun_call(f: String, args: Vec<ast::Exp>) -> ast::TypedExp {
    let f_type = symbols::get(f).t;
    match f_type {
        types::Type::Int | types::Type::Long => panic!("tried to use variable as function name."),
        types::Type::FunType {
            param_types,
            ret_type,
        } => {
            if param_types.len() != args.len() {
                panic!("function called with wrong number of arguments.")
            }
            let mut converted_args = vec![];
            for i in 0..param_types.len() {
                converted_args.push(convert_to(
                    Box::new(typecheck_exp(args[i]).e),
                    *param_types[i],
                ));
            }
            let call_exp = ast::Exp::FunCall {
                f: f,
                args: converted_args,
            };
            type_utils::set_type(call_exp, ret_type)
        }
    }
}

pub fn to_static_init(var_type: types::Type, init: ast::Exp) -> ast::TypedExp {
    match init {
        ast::Exp::Constant(c) => {
            let init_val = match const_convert::const_convert(var_type, c) {
                constants::T::ConstInt(i) => initializers::StaticInit::IntInit(i),
                constants::T::ConstLong(l) => initializers::StaticInit::LongInit(l),
            };
            symbols::InitialValue::Initial(init_val)
        }
        _ => panic!("non-constant initializer on static variable."),
    }
}

pub fn typecheck_block(
    ret_type: types::Type,
    b: ast::Block<ast::Exp>,
) -> ast::Block<ast::TypedExp> {
    match b {
        ast::Block::Block(block_items) => {
            let mut typed_block_items = vec![];
            for item in block_items {
                typed_block_items.push(typecheck_block_item(ret_type, item));
            }
        }
    }
}

pub fn typecheck_block_item(
    ret_type: types::Type,
    block_item: ast::BlockItem<ast::Exp>,
) -> ast::BlockItem<ast::TypedExp> {
    match block_item {
        ast::BlockItem::S(s) => typecheck_statement(ret_type, s),
        ast::BlockItem::D(d) => typecheck_local_decl(d),
    }
}

pub fn typecheck_statement(
    ret_type: types::Type,
    statement: ast::Statement<ast::Exp>,
) -> ast::Statement<ast::TypedExp> {
    match statement {
        ast::Statement::Return(e) => {
            let typed_e = typecheck_exp(e);
            ast::Statement::Return(convert_to(typed_e, ret_type))
        }
        ast::Statement::Expression(e) => ast::Statement::Expression(typecheck_exp(e)),
        ast::Statement::If {
            condition,
            then_clause,
            else_clause,
        } => ast::Statement::If {
            condition: typecheck_exp(condition),
            then_clause: typecheck_statement(ret_type, then_clause),
            else_clause: if else_clause.is_some() {
                typecheck_statement(ret_type, else_clause.unwrap())
            } else {
                None
            },
        },
        ast::Statement::Compound(block) => {
            ast::Statement::Compound(typecheck_block(ret_type, block))
        }
        ast::Statement::While {
            condition,
            body,
            id,
        } => ast::Statement::While {
            condition: typecheck_exp(condition),
            body: typecheck_statement(ret_type, body),
            id: id,
        },
        ast::Statement::DoWhile {
            body,
            condition,
            id,
        } => ast::Statement::DoWhile {
            body: typecheck_statement(ret_type, body),
            condition: typecheck_exp(condition),
            id: id,
        },
        ast::Statement::For {
            init,
            condition,
            post,
            body,
            id: _,
        } => {
            let typechecked_for_init = match init {
                ast::ForInit::InitDecl(ast::VariableDeclaration {
                    name: _,
                    var_type: _,
                    init: _,
                    storage_class: Some(_),
                }) => panic!("storage class not permitted on declaration in for loop header."),
                ast::ForInit::InitDecl(d) => ast::ForInit::InitDecl(typecheck_local_var_decl(d)),
                ast::ForInit::InitExp(e) => ast::ForInit::InitExp(if e.is_some() {
                    Some(typecheck_exp(e))
                } else {
                    None
                }),
            };
            ast::Statement::For {
                init: typechecked_for_init,
                condition: if condition.is_some() {
                    Some(typecheck_exp(condition))
                } else {
                    None
                },
                post: if post.is_some() {
                    Some(typecheck_exp(post))
                } else {
                    None
                },
                body: typecheck_statement(ret_type, body),
                id: id,
            }
        }
        s @ (ast::Statement::Null | ast::Statement::Break(_) | ast::Statement::Continue(_)) => s,
    }
}

pub fn typecheck_local_decl(d: ast::Declaration<ast::Exp>) -> ast::Declaration<ast::TypedExp> {
    match d {
        ast::Declaration::VarDecl(vd) => typecheck_local_var_decl(vd),
        ast::Declaration::FunDecl(fd) => typecheck_fn_decl(fd),
    }
}

pub fn typecheck_local_var_decl(
    vd: ast::VariableDeclaration<ast::Exp>,
) -> ast::VariableDeclaration<ast::TypedExp> {
    match vd.storage_class {
        Some(ast::StorageClass::Extern) => {
            if vd.init.is_some() {
                panic!("initializer on local extern declaration");
            }
            match symbols::get_opt(vd.name.clone()) {
                Some(symbols::Entry { t, attrs: _ }) => {
                    if t != types::Type::Int {
                        panic!("function redeclared as variable");
                    }
                }
                None => symbols::add_static_var(
                    vd.name,
                    types::Type::Int,
                    true,
                    symbols::InitialValue::NoInitializer,
                ),
            }
        }
        Some(ast::StorageClass::Static) => {
            let ini = match vd.init {
                Some(ast::Exp::Constant(i)) => symbols::InitialValue::Initial(i),
                None => symbols::InitialValue::Initial(0),
                Some(_) => panic!("non-constant initializer on local static variable"),
            };
            symbols::add_static_var(vd.name, types::Type::Int, false, ini);
        }
        None => {
            symbols::add_automatic_var(vd.name, types::Type::Int);
            if vd.init.is_some() {
                typecheck_exp(vd.init.unwrap());
            }
        }
    }
}

pub fn typecheck_fn_decl(
    fd: ast::FunctionDeclaration<ast::Exp>,
) -> ast::FunctionDeclaration<ast::TypedExp> {
    let has_body = fd.body.is_some();
    let global = fd.storage_class != Some(ast::StorageClass::Static);
    let old_decl = symbols::get_opt(fd.name.clone());
    let (defined, global) = match old_decl {
        None => (has_body, global),
        Some(_old_decl) => {
            if _old_decl.t != fd.fun_type {
                panic!("redeclared function {} with a different type", fd.name);
            } else {
                match _old_decl.attrs {
                    symbols::IdentifierAttrs::FunAttr {
                        defined: prev_defined,
                        global: prev_global,
                        stack_frame_size: _,
                    } => {
                        if prev_defined && has_body {
                            panic!("defined body of function {} twice.", fd.name.clone());
                        } else if prev_global && fd.storage_class == Some(ast::StorageClass::Static)
                        {
                            panic!("static function declaration follows non-static");
                        } else {
                            let defined = has_body || prev_defined;
                            (defined, prev_global)
                        }
                    }
                    _ => panic!("内部错误：symbol has function type but not function attributes."),
                }
            }
        }
    };

    symbols::add_fun(fd.name, fd.fun_type, global, defined);
    let (param_ts, return_t) = match fd.fun_type {
        types::Type::FunType {
            param_types,
            ret_type,
        } => (param_types, ret_type),
        _ => panic!("内部错误，function has non-function type."),
    };
    if has_body {
        for param in param_ts {
            symbols::add_automatic_var(param, types::Type::Int);
        }
    }
    if fd.body.is_some() {
        let body = typecheck_block(return_t, fd.body.unwrap());
    }
}

pub fn typecheck_file_scope_var_decl(
    vd: ast::VariableDeclaration<ast::Exp>,
) -> ast::VariableDeclaration<ast::TypedExp> {
    let current_init = match vd.init {
        Some(ast::Exp::Constant(c)) => symbols::InitialValue::Initial(c),
        None => {
            if vd.storage_class == Some(ast::StorageClass::Extern) {
                symbols::InitialValue::NoInitializer
            } else {
                symbols::InitialValue::Tentative
            }
        }
        Some(_) => panic!("file scope variable has non-constant initializer."),
    };
    let current_global = vd.storage_class != Some(ast::StorageClass::Extern);
    let old_decl = symbols::get_opt(vd.name.clone());
    let (global, init) = match old_decl {
        None => (current_global, current_init),
        Some(_old_decl) => {
            if _old_decl.t != types::Type::Int {
                panic!("function redeclared as variable.");
            } else {
                match _old_decl.attrs {
                    symbols::IdentifierAttrs::StaticAttr { init: prev_init, global: prev_global } => {
                        let global = if vd.storage_class == Some(ast::StorageClass::Extern) {
                            prev_global
                        } else if current_global == prev_global {
                            current_global
                        } else {
                            panic!("conflicting variable linkage.");
                        };
                        let init = match (prev_init.clone(), current_init.clone()) {
                            (symbols::InitialValue::Initial(_), symbols::InitialValue::Initial(_)) => panic!(),
                            (symbols::InitialValue::Initial(_), _) => prev_init,
                            (symbols::InitialValue::Tentative, symbols::InitialValue::Tentative | symbols::InitialValue::NoInitializer) => symbols::InitialValue::Tentative,
                            (_, symbols::InitialValue::Initial(_)) | (symbols::InitialValue::NoInitializer, _) => current_init
                        };
                        (global, init)
                    }
                    _ => panic!("内部错误：file-scope variable previously declared as local variable or function.")
                }
            }
        }
    };
    symbols::add_static_var(vd.name, types::Type::Int, global, init);
}

pub fn typecheck_global_decl(d: ast::Declaration<ast::Exp>) -> ast::Declaration<ast::TypedExp> {
    match d {
        ast::Declaration::FunDecl(fd) => typecheck_fn_decl(fd),
        ast::Declaration::VarDecl(vd) => typecheck_file_scope_var_decl(vd),
    }
}

pub fn typecheck(program: ast::UntypedProgType) -> ast::TypedProgType {
    match program {
        ast::TypedProgType::Program(fn_decls) => {
            for fn_decl in fn_decls {
                typecheck_global_decl(fn_decl);
            }
        }
    }
}
