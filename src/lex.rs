use regex::Regex;

use crate::tokens;

fn id_to_tok(s: String) -> tokens::T {
    match s.as_str() {
        "int" => tokens::T::KWInt,
        "return" => tokens::T::KWReturn,
        "void" => tokens::T::KWVoid,
        other => tokens::T::Identifier(other.to_string()),
    }
}

fn lex_helper(chars: Vec<u8>) -> Vec<tokens::T> {
    if chars.len() == 0 {
        vec![]
    } else if chars[0] == b'{' {
        let mut a = vec![tokens::T::OpenBrace];
        a.extend(lex_helper(chars[1..].to_vec()));
        a
    } else if chars[0] == b'}' {
        let mut a = vec![tokens::T::CloseBrace];
        a.extend(lex_helper(chars[1..].to_vec()));
        a
    } else if chars[0] == b'(' {
        let mut a = vec![tokens::T::OpenParen];
        a.extend(lex_helper(chars[1..].to_vec()));
        a
    } else if chars[0] == b')' {
        let mut a = vec![tokens::T::CloseParen];
        a.extend(lex_helper(chars[1..].to_vec()));
        a
    } else if chars[0] == b';' {
        let mut a = vec![tokens::T::Semicolon];
        a.extend(lex_helper(chars[1..].to_vec()));
        a
    } else if chars[0].is_ascii_whitespace() {
        lex_helper(chars[1..].to_vec())
    } else if chars[0].is_ascii_digit() {
        lex_constant(chars)
    } else {
        lex_identifier(chars)
    }
}

fn lex_constant(input_chars: Vec<u8>) -> Vec<tokens::T> {
    let input = String::from_utf8(input_chars.clone()).unwrap();
    let id_regexp = Regex::new(r"[0-9]+\b").expect("正则表达式语法错误");
    if let Some(id_str) = id_regexp.find(&input) {
        let tok = tokens::T::Constant(id_str.as_str().parse::<i64>().unwrap());
        let remaining = input_chars[id_str.end()..].to_vec();
        let mut result: Vec<tokens::T> = vec![tok];
        result.extend(lex_helper(remaining));
        result
    } else {
        panic!("Lexer failure: input doesn't match id_regexp: {}", input);
    }
}

fn lex_identifier(input_chars: Vec<u8>) -> Vec<tokens::T> {
    let input = String::from_utf8(input_chars.clone()).unwrap();
    let id_regexp = Regex::new(r"[A-Za-z_][A-Za-z0-9_]*\b").expect("正则表达式语法错误");
    if let Some(id_str) = id_regexp.find(&input) {
        let tok = id_to_tok(id_str.as_str().to_string());
        let remaining = input_chars[id_str.end()..].to_vec();
        let mut result: Vec<tokens::T> = vec![tok];
        result.extend(lex_helper(remaining));
        result
    } else {
        panic!("Lexer failure: input doesn't match id_regexp: {}", input);
    }
}

pub fn lex(input: &str) -> Vec<tokens::T> {
    let input = input.as_bytes().to_vec();
    lex_helper(input)
}

#[test]
fn test_lex_helper() {
    let program = "
        int main(void) {
            return 0;
        }
    ";
    println!("{:?}", lex(program));
}
