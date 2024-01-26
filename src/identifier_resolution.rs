use std::collections::HashMap;

use crate::{ast, unique_ids};

#[derive(Clone, Debug, PartialEq)]
pub struct VarEntry {
    unique_name: String,
    from_current_scope: bool,
    has_linkage: bool,
}

fn copy_identifier_map(m: HashMap<String, VarEntry>) -> HashMap<String, VarEntry> {
    let mut new_map = HashMap::new();
    for (k, v) in m {
        new_map.insert(
            k,
            VarEntry {
                unique_name: v.unique_name,
                from_current_scope: false,
                has_linkage: v.has_linkage,
            },
        );
    }
    new_map
}

fn resolve_optional_exp(
    var_map: HashMap<String, VarEntry>,
    exp: Option<ast::Exp>,
) -> Option<ast::Exp> {
    match exp {
        Some(e) => Some(resolve_exp(var_map, e)),
        None => None,
    }
}

fn resolve_exp(id_map: HashMap<String, VarEntry>, exp: ast::Exp) -> ast::Exp {
    match exp {
        ast::Exp::Assignment(left, right) => {
            match *left {
                ast::Exp::Var(_) => (),
                _ => panic!("赋值语句的左边应该是表达式，实际上是：{:?}", left),
            }
            ast::Exp::Assignment(
                Box::new(resolve_exp(id_map.clone(), *left)),
                Box::new(resolve_exp(id_map, *right)),
            )
        }
        ast::Exp::Var(v) => {
            if let Some(_v) = id_map.get(&v) {
                ast::Exp::Var(_v.clone().unique_name)
            } else {
                panic!("未声明变量：{:?}", v)
            }
        }
        ast::Exp::Unary(op, e) => ast::Exp::Unary(op, Box::new(resolve_exp(id_map, *e))),
        ast::Exp::Binary(op, e1, e2) => ast::Exp::Binary(
            op,
            Box::new(resolve_exp(id_map.clone(), *e1)),
            Box::new(resolve_exp(id_map, *e2)),
        ),
        ast::Exp::Conditional {
            condition,
            then_result,
            else_result,
        } => ast::Exp::Conditional {
            condition: Box::new(resolve_exp(id_map.clone(), *condition)),
            then_result: Box::new(resolve_exp(id_map.clone(), *then_result)),
            else_result: Box::new(resolve_exp(id_map, *else_result)),
        },
        ast::Exp::FunCall { f, args } => {
            if let Some(fn_name) = id_map.get(&f) {
                let mut resolved_args = vec![];
                for arg in args {
                    resolved_args.push(resolve_exp(id_map.clone(), arg));
                }
                ast::Exp::FunCall {
                    f: fn_name.clone().unique_name,
                    args: resolved_args,
                }
            } else {
                panic!("未声明函数。")
            }
        }
        c @ ast::Exp::Constant(_) => c,
    }
}

fn resolve_local_var_helper(
    id_map: HashMap<String, VarEntry>,
    name: String,
) -> (HashMap<String, VarEntry>, String) {
    match id_map.get(&name) {
        Some(VarEntry {
            unique_name: _,
            from_current_scope: true,
            has_linkage: _,
        }) => {
            panic!("变量重复声明。");
        }
        _ => {
            let unique_name = unique_ids::make_label(name.clone());
            let mut new_map = id_map.clone();
            new_map.insert(
                name,
                VarEntry {
                    unique_name: unique_name.clone(),
                    from_current_scope: true,
                    has_linkage: false,
                },
            );
            (new_map, unique_name)
        }
    }
}

fn resolve_local_var_declaration(
    id_map: HashMap<String, VarEntry>,
    vd: ast::VariableDeclaration,
) -> (HashMap<String, VarEntry>, ast::VariableDeclaration) {
    let (new_map, unique_name) = resolve_local_var_helper(id_map, vd.name);
    let resolved_init = match vd.init {
        Some(_init) => Some(resolve_exp(new_map.clone(), _init)),
        None => None,
    };
    (
        new_map,
        ast::VariableDeclaration {
            name: unique_name,
            init: resolved_init,
        },
    )
}

fn resolve_for_init(
    id_map: HashMap<String, VarEntry>,
    init: ast::ForInit,
) -> (HashMap<String, VarEntry>, ast::ForInit) {
    match init {
        ast::ForInit::InitExp(e) => (
            id_map.clone(),
            ast::ForInit::InitExp(resolve_optional_exp(id_map, e)),
        ),
        ast::ForInit::InitDecl(d) => {
            let (new_map, resolved_decl) = resolve_local_var_declaration(id_map, d);
            (new_map, ast::ForInit::InitDecl(resolved_decl))
        }
    }
}

fn resolve_statement(
    id_map: HashMap<String, VarEntry>,
    statement: ast::Statement,
) -> ast::Statement {
    match statement {
        ast::Statement::Return(e) => ast::Statement::Return(resolve_exp(id_map, e)),
        ast::Statement::Expression(e) => ast::Statement::Expression(resolve_exp(id_map, e)),
        ast::Statement::While {
            condition,
            body,
            id,
        } => ast::Statement::While {
            condition: resolve_exp(id_map.clone(), condition),
            body: Box::new(resolve_statement(id_map, *body)),
            id: id,
        },
        ast::Statement::DoWhile {
            body,
            condition,
            id,
        } => ast::Statement::DoWhile {
            body: Box::new(resolve_statement(id_map.clone(), *body)),
            condition: resolve_exp(id_map, condition),
            id: id,
        },
        ast::Statement::For {
            init,
            condition,
            post,
            body,
            id,
        } => {
            let id_map1 = copy_identifier_map(id_map);
            let (id_map2, resolved_init) = resolve_for_init(id_map1, init);
            ast::Statement::For {
                init: resolved_init,
                condition: resolve_optional_exp(id_map2.clone(), condition),
                post: resolve_optional_exp(id_map2.clone(), post),
                body: Box::new(resolve_statement(id_map2, *body)),
                id: id,
            }
        }
        ast::Statement::If {
            condition,
            then_clause,
            else_clause,
        } => ast::Statement::If {
            condition: resolve_exp(id_map.clone(), condition),
            then_clause: Box::new(resolve_statement(id_map.clone(), *then_clause)),
            else_clause: match else_clause {
                Some(_else_clause) => Some(Box::new(resolve_statement(id_map, *_else_clause))),
                None => None,
            },
        },
        ast::Statement::Compound(block) => {
            let new_variable_map = copy_identifier_map(id_map);
            ast::Statement::Compound(resolve_block(new_variable_map, block))
        }
        s @ (ast::Statement::Null | ast::Statement::Break(_) | ast::Statement::Continue(_)) => s,
    }
}

fn resolve_block_item(
    id_map: HashMap<String, VarEntry>,
    block_item: ast::BlockItem,
) -> (HashMap<String, VarEntry>, ast::BlockItem) {
    match block_item {
        ast::BlockItem::S(s) => {
            let resolved_s = resolve_statement(id_map.clone(), s);
            (id_map, ast::BlockItem::S(resolved_s))
        }
        ast::BlockItem::D(d) => {
            let (new_map, resolved_d) = resolve_local_declaration(id_map, d);
            (new_map, ast::BlockItem::D(resolved_d))
        }
    }
}

fn resolve_block(mut id_map: HashMap<String, VarEntry>, block: ast::Block) -> ast::Block {
    match block {
        ast::Block::Block(items) => {
            let mut resolved_items = vec![];
            for item in items {
                let t = resolve_block_item(id_map, item);
                id_map = t.0;
                resolved_items.push(t.1);
            }
            ast::Block::Block(resolved_items)
        }
    }
}

fn resolve_local_declaration(
    id_map: HashMap<String, VarEntry>,
    declaration: ast::Declaration,
) -> (HashMap<String, VarEntry>, ast::Declaration) {
    match declaration {
        ast::Declaration::VarDecl(vd) => {
            let (new_map, resolved_vd) = resolve_local_var_declaration(id_map, vd);
            (new_map, ast::Declaration::VarDecl(resolved_vd))
        }
        ast::Declaration::FunDecl(ast::FunctionDeclaration {
            name: _,
            params: _,
            body: Some(_),
        }) => {
            panic!("C语言不允许定义嵌套函数。");
        }
        ast::Declaration::FunDecl(fd) => {
            let (new_map, resolved_fd) = resolve_function_declaration(id_map, fd);
            (new_map, ast::Declaration::FunDecl(resolved_fd))
        }
    }
}

fn resolve_params(
    id_map: HashMap<String, VarEntry>,
    params: Vec<String>,
) -> (HashMap<String, VarEntry>, Vec<String>) {
    let mut new_map = id_map.clone();
    let mut resolved_params = vec![];
    for param in params {
        let t = resolve_local_var_helper(new_map, param);
        new_map = t.0;
        resolved_params.push(t.1);
    }
    (new_map, resolved_params)
}

fn resolve_function_declaration(
    id_map: HashMap<String, VarEntry>,
    f: ast::FunctionDeclaration,
) -> (HashMap<String, VarEntry>, ast::FunctionDeclaration) {
    match id_map.get(&f.name) {
        Some(VarEntry {
            unique_name: _,
            from_current_scope: true,
            has_linkage: false,
        }) => {
            panic!("函数重复声明。");
        }
        _ => {
            let new_entry = VarEntry {
                unique_name: f.name.clone(),
                from_current_scope: true,
                has_linkage: true,
            };
            let mut new_map = id_map.clone();
            new_map.insert(f.name.clone(), new_entry);
            let inner_map = copy_identifier_map(new_map.clone());
            let (inner_map1, resolved_params) = resolve_params(inner_map, f.params);
            let resolved_body = match f.body {
                Some(_body) => Some(resolve_block(inner_map1, _body)),
                None => None,
            };
            (
                new_map,
                ast::FunctionDeclaration {
                    name: f.name,
                    params: resolved_params,
                    body: resolved_body,
                },
            )
        }
    }
}

pub fn resolve(program: ast::Program) -> ast::Program {
    match program {
        ast::Program::FunctionDefinition(fn_defs) => {
            let mut resolved_decls = vec![];
            let mut id_map = HashMap::new();
            for fn_def in fn_defs {
                let t = resolve_function_declaration(id_map, fn_def);
                id_map = t.0;
                resolved_decls.push(t.1);
            }
            ast::Program::FunctionDefinition(resolved_decls)
        }
    }
}
