use crate::{ast, constants, tokens, types};

pub struct Parser {
    tokens: Vec<tokens::Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<tokens::Token>) -> Self {
        Parser {
            tokens: tokens,
            pos: 0,
        }
    }

    fn current_token(&mut self) -> tokens::Token {
        self.tokens[self.pos].clone()
    }

    fn eat_token(&mut self, expected: tokens::Token) {
        let actual = self.current_token();
        if actual != expected {
            panic!("预期token是: {:?}, 实际token是: {:?}", expected, actual);
        } else {
            self.pos += 1;
        }
    }

    fn get_precedence(&self, op: tokens::Token) -> Option<u8> {
        match op {
            tokens::Token::Star | tokens::Token::Slash | tokens::Token::Percent => Some(50),
            tokens::Token::Plus | tokens::Token::Hyphen => Some(45),
            tokens::Token::LessThan
            | tokens::Token::LessOrEqual
            | tokens::Token::GreaterThan
            | tokens::Token::GreaterOrEqual => Some(35),
            tokens::Token::DoubleEqual | tokens::Token::NotEqual => Some(30),
            tokens::Token::LogicalAnd => Some(10),
            tokens::Token::LogicalOr => Some(5),
            tokens::Token::QuestionMark => Some(3),
            tokens::Token::EqualSign => Some(1),
            _ => None,
        }
    }

    fn parse_type_specifier_list(&mut self) -> Vec<tokens::Token> {
        match self.current_token() {
            tokens::Token::KWInt | tokens::Token::KWLong => {
                let spec = self.current_token();
                self.pos += 1;
                let mut result = vec![spec];
                result.append(&mut self.parse_type_specifier_list());
                result
            }
            _ => vec![],
        }
    }

    fn parse_specifier_list(&mut self) -> Vec<tokens::Token> {
        match self.current_token() {
            tokens::Token::KWInt
            | tokens::Token::KWLong
            | tokens::Token::KWStatic
            | tokens::Token::KWExtern => {
                let spec = self.current_token();
                self.pos += 1;
                let mut result = vec![spec];
                result.append(&mut self.parse_specifier_list());
                result
            }
            _ => vec![],
        }
    }

    fn parse_storage_class(&mut self, storage_class: tokens::Token) -> ast::StorageClass {
        match storage_class {
            tokens::Token::KWExtern => ast::StorageClass::Extern,
            tokens::Token::KWStatic => ast::StorageClass::Static,
            _ => panic!("内部错误：bad storage class"),
        }
    }

    fn parse_type(&mut self, token_vec: Vec<tokens::Token>) -> types::Type {
        match token_vec.as_slice() {
            [tokens::Token::KWInt] => types::Type::Int,
            [tokens::Token::KWInt, tokens::Token::KWLong]
            | [tokens::Token::KWLong, tokens::Token::KWInt]
            | [tokens::Token::KWLong] => types::Type::Long,
            _ => panic!("invalid type specifier."),
        }
    }

    fn parse_type_and_storage_class(
        &mut self,
        specifier_list: Vec<tokens::Token>,
    ) -> (types::Type, Option<ast::StorageClass>) {
        let mut types = vec![];
        let mut storage_classes = vec![];
        for t in specifier_list {
            if t == tokens::Token::KWInt {
                types.push(t);
            } else {
                storage_classes.push(t);
            }
        }

        if types.len() != 1 {
            panic!("invalid type specifier.");
        } else {
            let storage_class = match storage_classes.len() {
                0 => None,
                1 => Some(self.parse_storage_class(storage_classes[0].clone())),
                _ => panic!(),
            };
            (types::Type::Int, storage_class)
        }
    }

    fn parse_id(&mut self) -> String {
        match self.current_token() {
            tokens::Token::Identifier(x) => {
                self.pos += 1;
                x
            }
            other => panic!("预期是identifer token,实际是{:?}", other),
        }
    }

    fn parse_constant(&mut self) -> ast::UnTypedExp {
        match self.current_token() {
            tokens::Token::ConstInt(c) => {
                self.pos += 1;
                ast::UnTypedExp::Constant(constants::T::ConstInt(c))
            }
            tokens::Token::ConstLong(c) => {
                self.pos += 1;
                ast::UnTypedExp::Constant(constants::T::ConstLong(c))
            }
            other => panic!("预期是常数 token,实际是{:?}", other),
        }
    }

    fn parse_unop(&mut self) -> ast::UnaryOperator {
        match self.current_token() {
            tokens::Token::Tilde => {
                self.pos += 1;
                ast::UnaryOperator::Complement
            }
            tokens::Token::Hyphen => {
                self.pos += 1;
                ast::UnaryOperator::Negate
            }
            tokens::Token::Bang => {
                self.pos += 1;
                ast::UnaryOperator::Not
            }
            other => panic!("预期是一元运算符 token,实际是{:?}", other),
        }
    }

    fn parse_binop(&mut self) -> ast::BinaryOperator {
        let current_token = self.current_token();
        self.pos += 1;
        match current_token {
            tokens::Token::Plus => ast::BinaryOperator::Add,
            tokens::Token::Hyphen => ast::BinaryOperator::Subtract,
            tokens::Token::Star => ast::BinaryOperator::Multiply,
            tokens::Token::Slash => ast::BinaryOperator::Divide,
            tokens::Token::Percent => ast::BinaryOperator::Mod,
            tokens::Token::LogicalAnd => ast::BinaryOperator::And,
            tokens::Token::LogicalOr => ast::BinaryOperator::Or,
            tokens::Token::DoubleEqual => ast::BinaryOperator::Equal,
            tokens::Token::NotEqual => ast::BinaryOperator::NotEqual,
            tokens::Token::LessThan => ast::BinaryOperator::LessThan,
            tokens::Token::LessOrEqual => ast::BinaryOperator::LessOrEqual,
            tokens::Token::GreaterThan => ast::BinaryOperator::GreaterThan,
            tokens::Token::GreaterOrEqual => ast::BinaryOperator::GreaterOrEqual,
            other => panic!("预期是二元运算符 token,实际是{:?}", other),
        }
    }

    /// <factor> ::= <int> | <identifier> <unop> <factor> | "(" <exp> ")"
    fn parse_factor(&mut self) -> ast::UnTypedExp {
        match self.current_token() {
            tokens::Token::ConstInt(_) | tokens::Token::ConstLong(_) => self.parse_constant(),
            tokens::Token::Identifier(_) => {
                let id = self.parse_id();
                match self.current_token() {
                    tokens::Token::OpenParen => {
                        let args = self.parse_optional_arg_list();
                        ast::UnTypedExp::FunCall { f: id, args: args }
                    }
                    _ => ast::UnTypedExp::Var(id),
                }
            }
            tokens::Token::Hyphen | tokens::Token::Tilde | tokens::Token::Bang => {
                let operator = self.parse_unop();
                let inner_exp = self.parse_factor();
                ast::UnTypedExp::Unary(operator, Box::new(inner_exp))
            }
            tokens::Token::OpenParen => {
                self.eat_token(tokens::Token::OpenParen); // 吃掉"(""
                let e = self.parse_expression(0);
                self.eat_token(tokens::Token::CloseParen); // 吃掉")"
                e
            }
            _ => panic!(
                "解析factor出错。碰到的token是：{:?}, {:?}",
                self.current_token(),
                self.tokens[self.pos + 1]
            ),
        }
    }

    fn parse_optional_arg_list(&mut self) -> Vec<ast::UnTypedExp> {
        self.eat_token(tokens::Token::OpenParen);
        let args = match self.current_token() {
            tokens::Token::CloseParen => vec![],
            _ => self.parse_arg_list(),
        };
        self.eat_token(tokens::Token::CloseParen);
        args
    }

    fn parse_arg_list(&mut self) -> Vec<ast::UnTypedExp> {
        let arg = self.parse_expression(0);
        match self.current_token() {
            tokens::Token::Comma => {
                self.eat_token(tokens::Token::Comma);
                let mut result = vec![];
                result.push(arg);
                result.append(&mut self.parse_arg_list());
                result
            }
            _ => vec![arg],
        }
    }

    /// "?" <exp> ":"
    fn parse_conditional_middle(&mut self) -> ast::UnTypedExp {
        self.eat_token(tokens::Token::QuestionMark);
        let e = self.parse_expression(0);
        self.eat_token(tokens::Token::Colon);
        e
    }

    fn parse_exp_loop(&mut self, left: ast::UnTypedExp, next: tokens::Token, min_prec: u8) -> ast::UnTypedExp {
        match self.get_precedence(next.clone()) {
            Some(prec) if prec >= min_prec => {
                if next == tokens::Token::EqualSign {
                    self.eat_token(tokens::Token::EqualSign);
                    let right = self.parse_expression(prec);
                    let left = ast::UnTypedExp::Assignment(Box::new(left), Box::new(right));
                    let peek_token = self.current_token();

                    self.parse_exp_loop(left, peek_token, min_prec)
                } else if next == tokens::Token::QuestionMark {
                    let middle = self.parse_conditional_middle();
                    let right = self.parse_expression(prec);
                    let left = ast::UnTypedExp::Conditional {
                        condition: Box::new(left),
                        then_result: Box::new(middle),
                        else_result: Box::new(right),
                    };
                    let peek_token = self.current_token();
                    self.parse_exp_loop(left, peek_token, min_prec)
                } else {
                    let operator = self.parse_binop();
                    let right = self.parse_expression(prec + 1);
                    let left = ast::UnTypedExp::Binary(operator, Box::new(left), Box::new(right));
                    let peek_token = self.current_token();
                    self.parse_exp_loop(left, peek_token, min_prec)
                }
            }
            _ => left,
        }
    }

    /// <exp> ::= <factor> | <exp> <binop> <exp> | <exp> "?" <exp> ":" <exp>
    fn parse_expression(&mut self, min_prec: u8) -> ast::UnTypedExp {
        let initial_factor = self.parse_factor();
        let next_token = self.current_token();
        self.parse_exp_loop(initial_factor, next_token, min_prec)
    }

    fn parse_optional_expression(&mut self, delim: tokens::Token) -> Option<ast::UnTypedExp> {
        if self.current_token() == delim {
            self.eat_token(delim);
            None
        } else {
            let e = self.parse_expression(0);
            self.eat_token(delim);
            Some(e)
        }
    }

    /// <statement> ::= "return" <exp> ";"
    ///               | <exp> ";"
    ///               | "if" "(" <exp> ")" <statement> [ "else" <statement> ]
    ///               | <block>
    ///               | "break" ";"
    ///               | "continue" ";"
    ///               | "while" "(" <exp> ")" <statement>
    ///               | "do" <statement> "while" "(" <exp> ")" ";"
    ///               | "for" "(" <for-init> [ <exp> ] ";" [ <exp> ] ")" <statement>
    ///               | ";"
    fn parse_statement(&mut self) -> ast::Statement<ast::UnTypedExp> {
        match self.current_token() {
            tokens::Token::KWIf => self.parse_if_statement(),
            tokens::Token::OpenBrace => ast::Statement::Compound(self.parse_block()),
            tokens::Token::KWDo => self.parse_do_loop(),
            tokens::Token::KWWhile => self.parse_while_loop(),
            tokens::Token::KWFor => self.parse_for_loop(),
            tokens::Token::KWBreak => {
                self.eat_token(tokens::Token::KWBreak);
                self.eat_token(tokens::Token::Semicolon);
                ast::Statement::Break("".to_string())
            }
            tokens::Token::KWContinue => {
                self.eat_token(tokens::Token::KWContinue);
                self.eat_token(tokens::Token::Semicolon);
                ast::Statement::Continue("".to_string())
            }
            tokens::Token::KWReturn => {
                self.eat_token(tokens::Token::KWReturn); // 吃掉"return"
                let exp = self.parse_expression(0);
                self.eat_token(tokens::Token::Semicolon); // 吃掉";"
                ast::Statement::Return(exp)
            }
            _ => {
                let opt_exp = self.parse_optional_expression(tokens::Token::Semicolon);
                match opt_exp {
                    Some(exp) => ast::Statement::Expression(exp),
                    None => ast::Statement::Null,
                }
            }
        }
    }

    /// "if" "(" <exp> ")" <statement> [ "else" <statement> ]
    fn parse_if_statement(&mut self) -> ast::Statement<ast::UnTypedExp> {
        self.eat_token(tokens::Token::KWIf);
        self.eat_token(tokens::Token::OpenParen);
        let condition = self.parse_expression(0);
        self.eat_token(tokens::Token::CloseParen);
        let then_clause = self.parse_statement();
        let else_clause = match self.current_token() {
            tokens::Token::KWElse => {
                self.pos += 1;
                Some(self.parse_statement())
            }
            _ => None,
        };
        ast::Statement::If {
            condition: condition,
            then_clause: Box::new(then_clause),
            else_clause: match else_clause {
                None => None,
                Some(_else_clause) => Some(Box::new(_else_clause)),
            },
        }
    }

    /// "do" <statement> "while" "(" <exp> ")" ";"
    fn parse_do_loop(&mut self) -> ast::Statement<ast::UnTypedExp> {
        self.eat_token(tokens::Token::KWDo);
        let body = self.parse_statement();
        self.eat_token(tokens::Token::KWWhile);
        self.eat_token(tokens::Token::OpenParen);
        let condition = self.parse_expression(0);
        self.eat_token(tokens::Token::CloseParen);
        self.eat_token(tokens::Token::Semicolon);
        ast::Statement::DoWhile {
            body: Box::new(body),
            condition: condition,
            id: "".to_string(),
        }
    }

    /// "while" "(" <exp> ")" <statement>
    fn parse_while_loop(&mut self) -> ast::Statement<ast::UnTypedExp> {
        self.eat_token(tokens::Token::KWWhile);
        self.eat_token(tokens::Token::OpenParen);
        let condition = self.parse_expression(0);
        self.eat_token(tokens::Token::CloseParen);
        let body = self.parse_statement();
        ast::Statement::While {
            condition: condition,
            body: Box::new(body),
            id: "".to_string(),
        }
    }

    /// "for" "(" <for-init> [ <exp> ] ";" [ <exp> ] ")" <statement>
    fn parse_for_loop(&mut self) -> ast::Statement<ast::UnTypedExp> {
        self.eat_token(tokens::Token::KWFor);
        self.eat_token(tokens::Token::OpenParen);
        let init = self.parse_for_init();
        let condition = self.parse_optional_expression(tokens::Token::Semicolon);
        let post = self.parse_optional_expression(tokens::Token::CloseParen);
        let body = self.parse_statement();
        ast::Statement::For {
            init: init,
            condition: condition,
            post: post,
            body: Box::new(body),
            id: "".to_string(),
        }
    }

    /// <block-item> ::= <statement> | <declaration>
    fn parse_block_item(&mut self) -> ast::BlockItem<ast::UnTypedExp> {
        match self.current_token() {
            tokens::Token::KWInt | tokens::Token::KWStatic | tokens::Token::KWExtern => {
                ast::BlockItem::D(self.parse_declaration())
            }
            _ => ast::BlockItem::S(self.parse_statement()),
        }
    }

    fn parse_block_item_list(&mut self) -> Vec<ast::BlockItem<ast::UnTypedExp>> {
        match self.current_token() {
            tokens::Token::CloseBrace => {
                vec![]
            }
            _ => {
                let next_block_item = self.parse_block_item();
                let mut result = vec![next_block_item];
                result.append(&mut self.parse_block_item_list());
                result
            }
        }
    }

    /// <block> ::= "{" { <block-item> } "}"
    fn parse_block(&mut self) -> ast::Block<ast::UnTypedExp> {
        self.eat_token(tokens::Token::OpenBrace);
        let block_items = self.parse_block_item_list();
        self.eat_token(tokens::Token::CloseBrace);
        ast::Block::Block(block_items)
    }

    fn finish_parsing_function_declaration(
        &mut self,
        ret_type: types::Type,
        storage_class: Option<ast::StorageClass>,
        name: String,
    ) -> ast::FunctionDeclaration<ast::UnTypedExp> {
        self.eat_token(tokens::Token::OpenParen);
        let params_with_types = match self.current_token() {
            tokens::Token::KWVoid => {
                self.eat_token(tokens::Token::KWVoid);
                vec![]
            }
            _ => self.parse_param_list(),
        };
        let mut param_types = vec![];
        let mut params = vec![];
        for p in params_with_types {
            param_types.push(p.0);
            params.push(p.1);
        }
        self.eat_token(tokens::Token::CloseParen);
        let body = match self.current_token() {
            tokens::Token::OpenBrace => Some(self.parse_block()),
            tokens::Token::Semicolon => {
                self.eat_token(tokens::Token::Semicolon);
                None
            }
            other => panic!("预期是函数体或者分号，实际上是：{:?}", other),
        };
        let fun_type = types::Type::FunType {
            param_types: param_types,
            ret_type: ret_type,
        };
        ast::FunctionDeclaration {
            name: name,
            fun_type: fun_type,
            params: params,
            body: body,
            storage_class: storage_class,
        }
    }

    fn parse_param_list(&mut self) -> Vec<(types::Type, String)> {
        let specifiers = self.parse_type_specifier_list();
        let param_type = self.parse_type(specifiers);
        let next_param = self.parse_id();
        match self.current_token() {
            tokens::Token::Comma => {
                self.eat_token(tokens::Token::Comma);
                let mut result = vec![];
                result.push((param_type, next_param));
                result.append(&mut self.parse_param_list());
                result
            }
            _ => vec![(param_type, next_param)],
        }
    }

    fn finish_parsing_variable_declaration(
        &mut self,
        var_type: types::Type,
        storage_class: Option<ast::StorageClass>,
        name: String,
    ) -> ast::VariableDeclaration<ast::UnTypedExp> {
        match self.current_token() {
            tokens::Token::Semicolon => ast::VariableDeclaration {
                name: name,
                var_type: var_type,
                init: None,
                storage_class: storage_class,
            },
            tokens::Token::EqualSign => {
                self.eat_token(tokens::Token::EqualSign);
                let init = self.parse_expression(0);
                self.eat_token(tokens::Token::Semicolon);
                ast::VariableDeclaration {
                    name: name,
                    var_type: var_type,
                    init: Some(init),
                    storage_class: storage_class,
                }
            }
            other => panic!("预期是一个变量初始化器后者分号，实际上是：{:?}", other),
        }
    }

    fn parse_declaration(&mut self) -> ast::Declaration<ast::UnTypedExp> {
        let specifiers = self.parse_specifier_list();
        let (_typ, storage_class) = self.parse_type_and_storage_class(specifiers);
        let name = self.parse_id();
        match self.current_token() {
            tokens::Token::OpenParen => ast::Declaration::FunDecl(
                self.finish_parsing_function_declaration(_typ, storage_class, name),
            ),
            _ => ast::Declaration::VarDecl(
                self.finish_parsing_variable_declaration(_typ, storage_class, name),
            ),
        }
    }

    fn parse_variable_declaration(&mut self) -> ast::VariableDeclaration<ast::UnTypedExp> {
        match self.parse_declaration() {
            ast::Declaration::VarDecl(vd) => vd,
            ast::Declaration::FunDecl(_) => panic!("预期是变量声明，这里是函数声明。"),
        }
    }

    /// <for-init> ::= <declaration> | [ <exp> ] ";"
    fn parse_for_init(&mut self) -> ast::ForInit<ast::UnTypedExp> {
        match self.current_token() {
            tokens::Token::KWInt | tokens::Token::KWStatic | tokens::Token::KWExtern => {
                ast::ForInit::InitDecl(self.parse_variable_declaration())
            }
            _ => {
                let opt_e = self.parse_optional_expression(tokens::Token::Semicolon);
                ast::ForInit::InitExp(opt_e)
            }
        }
    }

    /// <function> ::= "int" <identifier> "(" "void" ")" "{" { <block-item> } "}"
    fn parse_declaration_list(&mut self) -> Vec<ast::Declaration<ast::UnTypedExp>> {
        match self.current_token() {
            tokens::Token::Eof => vec![],
            _ => {
                let next_decl = self.parse_declaration();
                let mut result = vec![next_decl];
                result.append(&mut self.parse_declaration_list());
                result
            }
        }
    }

    /// <program> ::= <function>
    pub fn parse(&mut self) -> ast::UntypedProgType {
        let declarations = self.parse_declaration_list();
        ast::UntypedProgType::Program(declarations)
    }
}
