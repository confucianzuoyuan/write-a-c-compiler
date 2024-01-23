use std::collections::HashMap;

use crate::{ast, unique_ids};

#[derive(Clone, Debug, PartialEq)]
pub struct VarEntry {
    unique_name: String,
    from_current_block: bool,
}

fn copy_variable_map(m: HashMap<String, VarEntry>) -> HashMap<String, VarEntry> {
    let mut new_map = HashMap::new();
    for (k, v) in m {
        new_map.insert(
            k,
            VarEntry {
                unique_name: v.unique_name,
                from_current_block: false,
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

fn resolve_exp(var_map: HashMap<String, VarEntry>, exp: ast::Exp) -> ast::Exp {
    match exp {
        ast::Exp::Assignment(left, right) => {
            match *left {
                ast::Exp::Var(_) => (),
                _ => panic!("赋值语句的左边应该是表达式，实际上是：{:?}", left),
            }
            ast::Exp::Assignment(
                Box::new(resolve_exp(var_map.clone(), *left)),
                Box::new(resolve_exp(var_map, *right)),
            )
        }
        ast::Exp::Var(v) => {
            if let Some(_v) = var_map.get(&v) {
                ast::Exp::Var(_v.clone().unique_name)
            } else {
                panic!("未声明变量：{:?}", v)
            }
        }
        ast::Exp::Unary(op, e) => ast::Exp::Unary(op, Box::new(resolve_exp(var_map, *e))),
        ast::Exp::Binary(op, e1, e2) => ast::Exp::Binary(
            op,
            Box::new(resolve_exp(var_map.clone(), *e1)),
            Box::new(resolve_exp(var_map, *e2)),
        ),
        ast::Exp::Conditional {
            condition,
            then_result,
            else_result,
        } => ast::Exp::Conditional {
            condition: Box::new(resolve_exp(var_map.clone(), *condition)),
            then_result: Box::new(resolve_exp(var_map.clone(), *then_result)),
            else_result: Box::new(resolve_exp(var_map, *else_result)),
        },
        c @ ast::Exp::Constant(_) => c,
    }
}

fn resolve_declaration(
    var_map: HashMap<String, VarEntry>,
    declaration: ast::Declaration,
) -> (HashMap<String, VarEntry>, ast::Declaration) {
    if let Some(VarEntry {
        unique_name: _,
        from_current_block: true,
    }) = var_map.get(&declaration.name)
    {
        panic!("变量重复声明。");
    } else {
        let unique_name = unique_ids::make_label(declaration.name.clone());
        let mut new_map = var_map.clone();
        new_map.insert(
            declaration.name,
            VarEntry {
                unique_name: unique_name.clone(),
                from_current_block: true,
            },
        );
        let resolved_init = match declaration.init {
            Some(init) => Some(resolve_exp(new_map.clone(), init)),
            None => None,
        };
        (
            new_map,
            ast::Declaration {
                name: unique_name,
                init: resolved_init,
            },
        )
    }
}

fn resolve_for_init(
    var_map: HashMap<String, VarEntry>,
    init: ast::ForInit,
) -> (HashMap<String, VarEntry>, ast::ForInit) {
    match init {
        ast::ForInit::InitExp(e) => (
            var_map.clone(),
            ast::ForInit::InitExp(resolve_optional_exp(var_map, e)),
        ),
        ast::ForInit::InitDecl(d) => {
            let (new_map, resolved_decl) = resolve_declaration(var_map, d);
            (new_map, ast::ForInit::InitDecl(resolved_decl))
        }
    }
}

fn resolve_statement(
    var_map: HashMap<String, VarEntry>,
    statement: ast::Statement,
) -> ast::Statement {
    match statement {
        ast::Statement::Return(e) => ast::Statement::Return(resolve_exp(var_map, e)),
        ast::Statement::Expression(e) => ast::Statement::Expression(resolve_exp(var_map, e)),
        ast::Statement::While {
            condition,
            body,
            id,
        } => ast::Statement::While {
            condition: resolve_exp(var_map.clone(), condition),
            body: Box::new(resolve_statement(var_map, *body)),
            id: id,
        },
        ast::Statement::DoWhile {
            body,
            condition,
            id,
        } => ast::Statement::DoWhile {
            body: Box::new(resolve_statement(var_map.clone(), *body)),
            condition: resolve_exp(var_map, condition),
            id: id,
        },
        ast::Statement::For {
            init,
            condition,
            post,
            body,
            id,
        } => {
            let var_map1 = copy_variable_map(var_map);
            let (var_map2, resolved_init) = resolve_for_init(var_map1, init);
            ast::Statement::For {
                init: resolved_init,
                condition: resolve_optional_exp(var_map2.clone(), condition),
                post: resolve_optional_exp(var_map2.clone(), post),
                body: Box::new(resolve_statement(var_map2, *body)),
                id: id,
            }
        }
        ast::Statement::If {
            condition,
            then_clause,
            else_clause,
        } => ast::Statement::If {
            condition: resolve_exp(var_map.clone(), condition),
            then_clause: Box::new(resolve_statement(var_map.clone(), *then_clause)),
            else_clause: match else_clause {
                Some(_else_clause) => Some(Box::new(resolve_statement(var_map, *_else_clause))),
                None => None,
            },
        },
        ast::Statement::Compound(block) => {
            let new_variable_map = copy_variable_map(var_map);
            ast::Statement::Compound(resolve_block(new_variable_map, block))
        }
        s @ (ast::Statement::Null | ast::Statement::Break(_) | ast::Statement::Continue(_)) => s,
    }
}

fn resolve_block_item(
    var_map: HashMap<String, VarEntry>,
    block_item: ast::BlockItem,
) -> (HashMap<String, VarEntry>, ast::BlockItem) {
    match block_item {
        ast::BlockItem::S(s) => {
            let resolved_s = resolve_statement(var_map.clone(), s);
            (var_map, ast::BlockItem::S(resolved_s))
        }
        ast::BlockItem::D(d) => {
            let (new_map, resolved_d) = resolve_declaration(var_map, d);
            (new_map, ast::BlockItem::D(resolved_d))
        }
    }
}

fn resolve_block(mut var_map: HashMap<String, VarEntry>, block: ast::Block) -> ast::Block {
    match block {
        ast::Block::Block(items) => {
            let mut resolved_items = vec![];
            for item in items {
                let t = resolve_block_item(var_map, item);
                var_map = t.0;
                resolved_items.push(t.1);
            }
            ast::Block::Block(resolved_items)
        }
    }
}

fn resolve_function_def(f: ast::FunctionDefinition) -> ast::FunctionDefinition {
    match f {
        ast::FunctionDefinition::Function { name, body } => {
            let resolved_body = resolve_block(HashMap::new(), body);
            ast::FunctionDefinition::Function {
                name: name,
                body: resolved_body,
            }
        }
    }
}

pub fn resolve(program: ast::Program) -> ast::Program {
    match program {
        ast::Program::FunctionDefinition(fn_def) => {
            ast::Program::FunctionDefinition(resolve_function_def(fn_def))
        }
    }
}
