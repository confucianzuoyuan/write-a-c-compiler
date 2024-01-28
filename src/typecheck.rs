use crate::{ast, symbols, types};

#[derive(Clone, Debug, PartialEq)]
pub struct TypeCheck {
    pub symbol_table: symbols::SymbolTable,
}

impl TypeCheck {
    pub fn new() -> Self {
        TypeCheck {
            symbol_table: symbols::SymbolTable::new(),
        }
    }

    pub fn typecheck_exp(&mut self, exp: ast::Exp) {
        match exp {
            ast::Exp::FunCall { f, args } => {
                let t = self.symbol_table.get(f).t.clone();
                match t {
                    types::Type::Int => panic!("试图将一个变量作为函数名使用"),
                    types::Type::FunType { param_count } => {
                        if args.len() != param_count {
                            panic!("传给函数的参数数量错误。");
                        } else {
                            for arg in args {
                                self.typecheck_exp(arg);
                            }
                        }
                    }
                }
            }
            ast::Exp::Var(v) => {
                let t = self.symbol_table.get(v).t.clone();
                match t {
                    types::Type::Int => (),
                    types::Type::FunType { param_count: _ } => panic!("试图将函数名作为变量使用。"),
                }
            }
            ast::Exp::Unary(_, inner) => self.typecheck_exp(*inner),
            ast::Exp::Binary(_, e1, e2) => {
                self.typecheck_exp(*e1);
                self.typecheck_exp(*e2);
            }
            ast::Exp::Assignment(lhs, rhs) => {
                self.typecheck_exp(*lhs);
                self.typecheck_exp(*rhs);
            }
            ast::Exp::Conditional {
                condition,
                then_result,
                else_result,
            } => {
                self.typecheck_exp(*condition);
                self.typecheck_exp(*then_result);
                self.typecheck_exp(*else_result);
            }
            ast::Exp::Constant(_) => (),
        }
    }

    pub fn typecheck_block(&mut self, b: ast::Block) {
        match b {
            ast::Block::Block(block_items) => {
                for item in block_items {
                    self.typecheck_block_item(item);
                }
            }
        }
    }

    pub fn typecheck_block_item(&mut self, block_item: ast::BlockItem) {
        match block_item {
            ast::BlockItem::S(s) => self.typecheck_statement(s),
            ast::BlockItem::D(d) => self.typecheck_decl(d),
        }
    }

    pub fn typecheck_statement(&mut self, statement: ast::Statement) {
        match statement {
            ast::Statement::Return(e) => self.typecheck_exp(e),
            ast::Statement::Expression(e) => self.typecheck_exp(e),
            ast::Statement::If {
                condition,
                then_clause,
                else_clause,
            } => {
                self.typecheck_exp(condition);
                self.typecheck_statement(*then_clause);
                if else_clause.is_some() {
                    self.typecheck_statement(*else_clause.unwrap());
                }
            }
            ast::Statement::Compound(block) => self.typecheck_block(block),
            ast::Statement::While {
                condition,
                body,
                id: _,
            } => {
                self.typecheck_exp(condition);
                self.typecheck_statement(*body);
            }
            ast::Statement::DoWhile {
                body,
                condition,
                id: _,
            } => {
                self.typecheck_statement(*body);
                self.typecheck_exp(condition);
            }
            ast::Statement::For {
                init,
                condition,
                post,
                body,
                id: _,
            } => {
                match init {
                    ast::ForInit::InitDecl(d) => self.typecheck_var_decl(d),
                    ast::ForInit::InitExp(e) => {
                        if e.is_some() {
                            self.typecheck_exp(e.unwrap());
                        }
                    }
                };
                if condition.is_some() {
                    self.typecheck_exp(condition.unwrap());
                }
                if post.is_some() {
                    self.typecheck_exp(post.unwrap());
                }
                self.typecheck_statement(*body);
            }
            ast::Statement::Null | ast::Statement::Break(_) | ast::Statement::Continue(_) => (),
        }
    }

    pub fn typecheck_decl(&mut self, d: ast::Declaration) {
        match d {
            ast::Declaration::VarDecl(vd) => self.typecheck_var_decl(vd),
            ast::Declaration::FunDecl(fd) => self.typecheck_fn_decl(fd),
        }
    }

    pub fn typecheck_var_decl(&mut self, vd: ast::VariableDeclaration) {
        self.symbol_table.add_var(vd.name, types::Type::Int);
        if vd.init.is_some() {
            self.typecheck_exp(vd.init.unwrap());
        }
    }

    pub fn typecheck_fn_decl(&mut self, fd: ast::FunctionDeclaration) {
        let fun_type = types::Type::FunType {
            param_count: fd.params.len(),
        };
        let has_body = fd.body.is_some();
        let old_decl = self.symbol_table.get_opt(fd.name.clone());
        if old_decl.is_some() {
            let _old_decl = (*old_decl.unwrap()).clone();
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
            }) => *is_defined,
            None => false,
        };
        self.symbol_table
            .add_fun(fd.name, fun_type, already_defined || has_body);
        if has_body {
            for param in fd.params {
                self.symbol_table.add_var(param, types::Type::Int);
            }
        }
        if fd.body.is_some() {
            self.typecheck_block(fd.body.unwrap());
        }
    }

    pub fn typecheck(&mut self, program: ast::Program) {
        match program {
            ast::Program::FunctionDefinition(fn_decls) => {
                for fn_decl in fn_decls {
                    self.typecheck_fn_decl(fn_decl);
                }
            }
        }
    }
}
