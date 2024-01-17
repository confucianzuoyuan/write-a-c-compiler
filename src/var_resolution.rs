use std::collections::HashMap;

use crate::{ast, unique_ids};

#[derive(Clone, Debug, PartialEq)]
pub struct VarResolution {
    var_map: HashMap<String, String>,
}

impl VarResolution {
    pub fn new() -> Self {
        VarResolution {
            var_map: HashMap::new(),
        }
    }

    fn resolve_exp(&mut self, exp: ast::Exp) -> ast::Exp {
        match exp {
            ast::Exp::Assignment(left, right) => {
                let _ = match *left {
                    ast::Exp::Var(_) => (),
                    _ => panic!("预期的表达式应该在赋值表达式左边，实际是：{:?}", left),
                };
                ast::Exp::Assignment(
                    Box::new(self.resolve_exp(*left)),
                    Box::new(self.resolve_exp(*right)),
                )
            }
            ast::Exp::Var(v) => {
                if let Some(_v) = self.var_map.get(&v) {
                    ast::Exp::Var(_v.clone())
                } else {
                    panic!("未声明变量。");
                }
            }
            ast::Exp::Unary(op, e) => ast::Exp::Unary(op, Box::new(self.resolve_exp(*e))),
            ast::Exp::Binary(op, e1, e2) => ast::Exp::Binary(
                op,
                Box::new(self.resolve_exp(*e1)),
                Box::new(self.resolve_exp(*e2)),
            ),
            c @ ast::Exp::Constant(_) => c,
        }
    }

    fn resolve_declaration(&mut self, declaration: ast::Declaration) -> ast::Declaration {
        if let Some(_) = self.var_map.get(&declaration.name) {
            panic!("变量重复声明了。");
        } else {
            let unique_name = unique_ids::make_label(declaration.name.clone());
            self.var_map.insert(declaration.name, unique_name.clone());
            let resolve_init = match declaration.init {
                Some(init) => Some(self.resolve_exp(init)),
                None => None,
            };
            ast::Declaration {
                name: unique_name,
                init: resolve_init,
            }
        }
    }

    fn resolve_statement(&mut self, statement: ast::Statement) -> ast::Statement {
        match statement {
            ast::Statement::Return(e) => ast::Statement::Return(self.resolve_exp(e)),
            ast::Statement::Expression(e) => ast::Statement::Expression(self.resolve_exp(e)),
            ast::Statement::Null => ast::Statement::Null,
        }
    }

    fn resolve_block_item(&mut self, block_item: ast::BlockItem) -> ast::BlockItem {
        match block_item {
            ast::BlockItem::S(s) => {
                let resolved_s = self.resolve_statement(s);
                ast::BlockItem::S(resolved_s)
            }
            ast::BlockItem::D(d) => {
                let resolved_d = self.resolve_declaration(d);
                ast::BlockItem::D(resolved_d)
            }
        }
    }

    fn resolve_function_def(&mut self, f: ast::FunctionDefinition) -> ast::FunctionDefinition {
        match f {
            ast::FunctionDefinition::Function { name, body } => {
                let mut resolved_body = vec![];
                for b in body {
                    resolved_body.push(self.resolve_block_item(b));
                }
                ast::FunctionDefinition::Function {
                    name: name,
                    body: resolved_body,
                }
            }
        }
    }

    pub fn resolve(&mut self, program: ast::Program) -> ast::Program {
        match program {
            ast::Program::FunctionDefinition(fn_def) => {
                ast::Program::FunctionDefinition(self.resolve_function_def(fn_def))
            }
        }
    }
}
