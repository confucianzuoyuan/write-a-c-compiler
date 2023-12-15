use crate::{ast, tokens};

fn expect(expected: tokens::T, tokens: &mut Vec<tokens::T>) {
    let actual = tokens.remove(0);
    if actual != expected {
        panic!("expected: {:?}, actual: {:?}", expected, actual);
    }
}

fn expect_empty(tokens: &mut Vec<tokens::T>) {
    if tokens.len() != 0 {
        let bad_token = tokens.remove(0);
        panic!("expected end of file, actual: {:?}", bad_token);
    }
}

fn parse_id(tokens: &mut Vec<tokens::T>) -> String {
    match tokens.remove(0) {
        tokens::T::Identifier(ident) => ident,
        other => panic!("expected an identifier, actual: {:?}", other),
    }
}

fn parse_expression(tokens: &mut Vec<tokens::T>) -> ast::Exp {
    match tokens.remove(0) {
        tokens::T::Constant(c) => ast::Exp::Constant { value: c },
        other => panic!("expected an expression, actual: {:?}", other),
    }
}

fn parse_statement(tokens: &mut Vec<tokens::T>) -> ast::Statement {
    expect(tokens::T::KWReturn, tokens);
    let exp = parse_expression(tokens);
    expect(tokens::T::Semicolon, tokens);
    ast::Statement::Return { exp: exp }
}

fn parse_function_definition(tokens: &mut Vec<tokens::T>) -> ast::FunctionDefinition {
    expect(tokens::T::KWInt, tokens);
    let fun_name = parse_id(tokens);
    expect(tokens::T::OpenParen, tokens);
    expect(tokens::T::KWVoid, tokens);
    expect(tokens::T::CloseParen, tokens);
    expect(tokens::T::OpenBrace, tokens);
    let statement = parse_statement(tokens);
    expect(tokens::T::CloseBrace, tokens);
    ast::FunctionDefinition::Function {
        name: fun_name,
        body: statement,
    }
}

pub fn parse(tokens: &mut Vec<tokens::T>) -> ast::T {
    let fun_def = parse_function_definition(tokens);
    expect_empty(tokens);
    ast::T::Program {
        function_definition: fun_def,
    }
}

#[test]
fn test_parse() {
    let mut tokens = vec![
        tokens::T::KWInt,
        tokens::T::Identifier("main".to_string()),
        tokens::T::OpenParen,
        tokens::T::KWVoid,
        tokens::T::CloseParen,
        tokens::T::OpenBrace,
        tokens::T::KWReturn,
        tokens::T::Constant(0),
        tokens::T::Semicolon,
        tokens::T::CloseBrace,
    ];

    let tree = parse(&mut tokens);
    println!("tree: {:?}", tree);
}
