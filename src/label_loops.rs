use crate::{ast, unique_ids};

fn label_statement(current_label: Option<String>, statement: ast::Statement) -> ast::Statement {
    match statement {
        ast::Statement::Break(_) => match current_label {
            Some(l) => ast::Statement::Break(l),
            None => panic!("break outside of loop"),
        },
        ast::Statement::Continue(_) => match current_label {
            Some(l) => ast::Statement::Continue(l),
            None => panic!("continue outside of loop"),
        },
        ast::Statement::While {
            condition,
            body,
            id: _,
        } => {
            let new_id = unique_ids::make_label("while".to_string());
            ast::Statement::While {
                condition: condition,
                body: Box::new(label_statement(Some(new_id.clone()), *body)),
                id: new_id,
            }
        }
        ast::Statement::DoWhile {
            body,
            condition,
            id: _,
        } => {
            let new_id = unique_ids::make_label("do_while".to_string());
            ast::Statement::DoWhile {
                body: Box::new(label_statement(Some(new_id.clone()), *body)),
                condition: condition,
                id: new_id,
            }
        }
        ast::Statement::For {
            init,
            condition,
            post,
            body,
            id: _,
        } => {
            let new_id = unique_ids::make_label("for".to_string());
            ast::Statement::For {
                init: init,
                condition: condition,
                post: post,
                body: Box::new(label_statement(Some(new_id.clone()), *body)),
                id: new_id,
            }
        }
        ast::Statement::Compound(blk) => ast::Statement::Compound(label_block(current_label, blk)),
        ast::Statement::If {
            condition,
            then_clause,
            else_clause,
        } => ast::Statement::If {
            condition: condition,
            then_clause: Box::new(label_statement(current_label.clone(), *then_clause)),
            else_clause: match else_clause {
                Some(_else_clause) => Some(Box::new(label_statement(current_label, *_else_clause))),
                None => None,
            },
        },
        s @ (ast::Statement::Null | ast::Statement::Return(_) | ast::Statement::Expression(_)) => s,
    }
}

fn label_block_item(current_label: Option<String>, block_item: ast::BlockItem) -> ast::BlockItem {
    match block_item {
        ast::BlockItem::S(s) => ast::BlockItem::S(label_statement(current_label, s)),
        decl => decl,
    }
}

fn label_block(current_label: Option<String>, b: ast::Block) -> ast::Block {
    match b {
        ast::Block::Block(items) => {
            let mut block_items = vec![];
            for item in items {
                block_items.push(label_block_item(current_label.clone(), item));
            }
            ast::Block::Block(block_items)
        }
    }
}

fn label_function_def(f: ast::FunctionDeclaration) -> ast::FunctionDeclaration {
    match f {
        ast::FunctionDeclaration { name, params, body } => ast::FunctionDeclaration {
            name: name,
            params: params,
            body: match body {
                Some(_body) => Some(label_block(None, _body)),
                None => None,
            },
        },
    }
}

pub fn label_loops(program: ast::Program) -> ast::Program {
    match program {
        ast::Program::FunctionDefinition(fn_defs) => {
            let mut arr = vec![];
            for fn_def in fn_defs {
                arr.push(label_function_def(fn_def));
            }
            ast::Program::FunctionDefinition(arr)
        }
    }
}