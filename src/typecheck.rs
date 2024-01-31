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
        ast::BlockItem::D(d) => typecheck_local_decl(d),
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
                ast::ForInit::InitDecl(d) => typecheck_local_var_decl(d),
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

pub fn typecheck_local_decl(d: ast::Declaration) {
    match d {
        ast::Declaration::VarDecl(vd) => typecheck_local_var_decl(vd),
        ast::Declaration::FunDecl(fd) => typecheck_fn_decl(fd),
    }
}

pub fn typecheck_local_var_decl(vd: ast::VariableDeclaration) {
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

pub fn typecheck_fn_decl(fd: ast::FunctionDeclaration) {
    let fun_type = types::Type::FunType {
        param_count: fd.params.len(),
    };
    let has_body = fd.body.is_some();
    let global = fd.storage_class != Some(ast::StorageClass::Static);
    let old_decl = symbols::get_opt(fd.name.clone());
    let (defined, global) = match old_decl {
        None => (has_body, global),
        Some(_old_decl) => {
            if _old_decl.t != fun_type {
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

    symbols::add_fun(fd.name, fun_type, global, defined);
    if has_body {
        for param in fd.params {
            symbols::add_automatic_var(param, types::Type::Int);
        }
    }
    if fd.body.is_some() {
        typecheck_block(fd.body.unwrap());
    }
}

pub fn typecheck_file_scope_var_decl(vd: ast::VariableDeclaration) {
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

pub fn typecheck_global_decl(d: ast::Declaration) {
    match d {
        ast::Declaration::FunDecl(fd) => typecheck_fn_decl(fd),
        ast::Declaration::VarDecl(vd) => typecheck_file_scope_var_decl(vd),
    }
}

pub fn typecheck(program: ast::T) {
    match program {
        ast::T::Program(fn_decls) => {
            for fn_decl in fn_decls {
                typecheck_global_decl(fn_decl);
            }
        }
    }
}
