use crate::{ast, symbols, types};

pub fn typecheck_exp(exp: ast::Exp) {
    match exp {
        ast::Exp::FunCall { f, args } => {
            let t = symbols::get(f).t.clone();
            match t {
                types::Type::Int => panic!("试图将一个变量作为函数名使用"),
                types::Type::FunType { param_count } => {
                    if args.len() != param_count {
                        panic!("传给函数的参数数量错误。");
                    } else {
                        for arg in args {
                            typecheck_exp(arg);
                        }
                    }
                }
            }
        }
        ast::Exp::Var(v) => {
            let t = symbols::get(v).t.clone();
            match t {
                types::Type::Int => (),
                types::Type::FunType { param_count: _ } => panic!("试图将函数名作为变量使用。"),
            }
        }
        ast::Exp::Unary(_, inner) => typecheck_exp(*inner),
        ast::Exp::Binary(_, e1, e2) => {
            typecheck_exp(*e1);
            typecheck_exp(*e2);
        }
        ast::Exp::Assignment(lhs, rhs) => {
            typecheck_exp(*lhs);
            typecheck_exp(*rhs);
        }
        ast::Exp::Conditional {
            condition,
            then_result,
            else_result,
        } => {
            typecheck_exp(*condition);
            typecheck_exp(*then_result);
            typecheck_exp(*else_result);
        }
        ast::Exp::Constant(_) => (),
    }
}

pub fn typecheck_block(b: ast::Block) {
    match b {
        ast::Block::Block(block_items) => {
            for item in block_items {
                typecheck_block_item(item);
            }
        }
    }
}

pub fn typecheck_block_item(block_item: ast::BlockItem) {
    match block_item {
        ast::BlockItem::S(s) => typecheck_statement(s),
        ast::BlockItem::D(d) => typecheck_decl(d),
    }
}

pub fn typecheck_statement(statement: ast::Statement) {
    match statement {
        ast::Statement::Return(e) => typecheck_exp(e),
        ast::Statement::Expression(e) => typecheck_exp(e),
        ast::Statement::If {
            condition,
            then_clause,
            else_clause,
        } => {
            typecheck_exp(condition);
            typecheck_statement(*then_clause);
            if else_clause.is_some() {
                typecheck_statement(*else_clause.unwrap());
            }
        }
        ast::Statement::Compound(block) => typecheck_block(block),
        ast::Statement::While {
            condition,
            body,
            id: _,
        } => {
            typecheck_exp(condition);
            typecheck_statement(*body);
        }
        ast::Statement::DoWhile {
            body,
            condition,
            id: _,
        } => {
            typecheck_statement(*body);
            typecheck_exp(condition);
        }
        ast::Statement::For {
            init,
            condition,
            post,
            body,
            id: _,
        } => {
            match init {
                ast::ForInit::InitDecl(d) => typecheck_var_decl(d),
                ast::ForInit::InitExp(e) => {
                    if e.is_some() {
                        typecheck_exp(e.unwrap());
                    }
                }
            };
            if condition.is_some() {
                typecheck_exp(condition.unwrap());
            }
            if post.is_some() {
                typecheck_exp(post.unwrap());
            }
            typecheck_statement(*body);
        }
        ast::Statement::Null | ast::Statement::Break(_) | ast::Statement::Continue(_) => (),
    }
}

pub fn typecheck_decl(d: ast::Declaration) {
    match d {
        ast::Declaration::VarDecl(vd) => typecheck_var_decl(vd),
        ast::Declaration::FunDecl(fd) => typecheck_fn_decl(fd),
    }
}

pub fn typecheck_var_decl(vd: ast::VariableDeclaration) {
    symbols::add_var(vd.name, types::Type::Int);
    if vd.init.is_some() {
        typecheck_exp(vd.init.unwrap());
    }
}

pub fn typecheck_fn_decl(fd: ast::FunctionDeclaration) {
    let fun_type = types::Type::FunType {
        param_count: fd.params.len(),
    };
    let has_body = fd.body.is_some();
    let old_decl = symbols::get_opt(fd.name.clone());
    if old_decl.is_some() {
        let _old_decl = old_decl.clone().unwrap();
        let prev_t = _old_decl.t;
        let is_defined = _old_decl.is_defined;
        if prev_t != fun_type {
            panic!("redeclared function {} with a different type.", fd.name);
        } else if is_defined && has_body {
            panic!("defined body of function {} twice", fd.name);
        }
    }
    let already_defined = match old_decl {
        Some(symbols::Entry {
            t: _,
            is_defined,
            stack_frame_size: _,
        }) => is_defined,
        None => false,
    };
    symbols::add_fun(fd.name, fun_type, already_defined || has_body);
    if has_body {
        for param in fd.params {
            symbols::add_var(param, types::Type::Int);
        }
    }
    if fd.body.is_some() {
        typecheck_block(fd.body.unwrap());
    }
}

pub fn typecheck(program: ast::Program) {
    match program {
        ast::Program::FunctionDefinition(fn_decls) => {
            for fn_decl in fn_decls {
                typecheck_fn_decl(fn_decl);
            }
        }
    }
}
