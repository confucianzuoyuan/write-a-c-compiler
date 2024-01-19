use std::{
    io::{Bytes, Read},
    iter::Peekable,
};

use crate::tokens;

pub struct Lexer<R: Read> {
    bytes_iter: Peekable<Bytes<R>>,
    pos: u64,
    saved_pos: u64,
}

impl<R: Read> Lexer<R> {
    pub fn new(reader: R) -> Self {
        Lexer {
            bytes_iter: reader.bytes().peekable(),
            pos: 0,
            saved_pos: 0,
        }
    }

    fn advance(&mut self) {
        self.bytes_iter.next();
        self.pos += 1;
    }

    fn current_pos(&self) -> u64 {
        self.pos
    }

    fn current_char(&mut self) -> Option<u8> {
        match self.bytes_iter.peek() {
            Some(&Ok(byte)) => Some(byte),
            None => None,
            _ => unreachable!(),
        }
    }

    fn eat(&mut self, ch: u8) {
        if self.current_char().unwrap() != ch {
            panic!(
                "预期字符是: `{}`, 但是当前字符是: `{}`。",
                ch,
                self.current_char().unwrap()
            );
        }
        self.advance()
    }

    fn save_start(&mut self) {
        self.saved_pos = self.current_pos();
    }

    fn identifier(&mut self) -> tokens::Token {
        self.save_start();
        let mut buffer = String::new();
        buffer.push(self.current_char().unwrap() as char);
        self.advance();
        let mut ch = self.current_char().unwrap();
        while ch.is_ascii_alphanumeric() || ch == b'_' {
            buffer.push(ch as char);
            self.advance();
            if let Some(c) = self.current_char() {
                ch = c;
            } else {
                break;
            }
        }
        let token = match buffer.as_str() {
            "void" => tokens::Token::KWVoid,
            "int" => tokens::Token::KWInt,
            "return" => tokens::Token::KWReturn,
            "if" => tokens::Token::KWIf,
            "else" => tokens::Token::KWElse,
            _ => tokens::Token::Identifier(buffer),
        };
        token
    }

    fn integer(&mut self) -> tokens::Token {
        self.save_start();
        let mut buffer = String::new();
        buffer.push(self.current_char().unwrap() as char);
        self.advance();
        let mut ch = self.current_char().unwrap();
        while ch.is_ascii_digit() {
            buffer.push(ch as char);
            self.advance();
            if let Some(c) = self.current_char() {
                ch = c;
            } else {
                break;
            }
        }
        let num = buffer.parse().unwrap();
        tokens::Token::Constant(num)
    }

    pub fn get_one_token(&mut self) -> tokens::Token {
        if let Some(&Ok(ch)) = self.bytes_iter.peek() {
            return match ch {
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.identifier(),
                b'0'..=b'9' => self.integer(),
                b' ' | b'\n' | b'\t' => {
                    self.advance();
                    self.get_one_token()
                }
                b'%' => {
                    self.advance();
                    tokens::Token::Percent
                }
                b';' => {
                    self.advance();
                    tokens::Token::Semicolon
                }
                b'+' => {
                    self.advance();
                    tokens::Token::Plus
                }
                b'*' => {
                    self.advance();
                    tokens::Token::Star
                }
                b'{' => {
                    self.advance();
                    tokens::Token::OpenBrace
                }
                b'}' => {
                    self.advance();
                    tokens::Token::CloseBrace
                }
                b'(' => {
                    self.advance();
                    tokens::Token::OpenParen
                }
                b')' => {
                    self.advance();
                    tokens::Token::CloseParen
                }
                b'~' => {
                    self.advance();
                    tokens::Token::Tilde
                }
                b'?' => {
                    self.advance();
                    tokens::Token::QuestionMark
                }
                b':' => {
                    self.advance();
                    tokens::Token::Colon
                }
                b'/' => {
                    self.advance();
                    tokens::Token::Slash
                }
                b'=' => {
                    self.advance();
                    if let Some(&Ok(b'=')) = self.bytes_iter.peek() {
                        self.advance();
                        tokens::Token::DoubleEqual
                    } else {
                        tokens::Token::EqualSign
                    }
                }
                b'&' => {
                    self.advance();
                    if let Some(&Ok(b'&')) = self.bytes_iter.peek() {
                        self.advance();
                        tokens::Token::LogicalAnd
                    } else {
                        panic!("目前不支持与符号`&`\r\n");
                    }
                }
                b'|' => {
                    self.advance();
                    if let Some(&Ok(b'|')) = self.bytes_iter.peek() {
                        self.advance();
                        tokens::Token::LogicalOr
                    } else {
                        panic!("目前不支持或符号`|`\r\n");
                    }
                }
                b'<' => {
                    self.advance();
                    if let Some(&Ok(b'=')) = self.bytes_iter.peek() {
                        self.advance();
                        tokens::Token::LessOrEqual
                    } else {
                        tokens::Token::LessThan
                    }
                }
                b'>' => {
                    self.advance();
                    if let Some(&Ok(b'=')) = self.bytes_iter.peek() {
                        self.advance();
                        tokens::Token::GreaterOrEqual
                    } else {
                        tokens::Token::GreaterThan
                    }
                }
                b'!' => {
                    self.advance();
                    if let Some(&Ok(b'=')) = self.bytes_iter.peek() {
                        self.advance();
                        tokens::Token::NotEqual
                    } else {
                        tokens::Token::Bang
                    }
                }
                b'-' => {
                    self.advance();
                    if let Some(&Ok(b'-')) = self.bytes_iter.peek() {
                        self.advance();
                        tokens::Token::DoubleHyphen
                    } else {
                        tokens::Token::Hyphen
                    }
                }
                _ => unreachable!(),
            };
        } else {
            tokens::Token::Eof
        }
    }

    pub fn lex(&mut self) -> Vec<tokens::Token> {
        let mut tokens = vec![];
        loop {
            let token = self.get_one_token();
            tokens.push(token.clone());
            if token == tokens::Token::Eof {
                break;
            }
        }
        tokens
    }
}

#[test]
fn test_1() {
    let prog = "int main() {return 100;}";
    let mut lexer = Lexer::new(prog.as_bytes());
    loop {
        let token = lexer.get_one_token();
        println!("{:?}", token);
        if token == tokens::Token::Eof {
            break;
        }
    }
}

#[test]
fn test_2() {
    let prog = "   int   main    (  )  {   return  0 ; }";
    let mut lexer = Lexer::new(prog.as_bytes());
    loop {
        let token = lexer.get_one_token();
        println!("{:?}", token);
        if token == tokens::Token::Eof {
            break;
        }
    }
}

#[test]
fn test_3() {
    let prog = "
    int main() {
        return 1 + 2;
    }
    ";
    let mut lexer = Lexer::new(prog.as_bytes());
    loop {
        let token = lexer.get_one_token();
        println!("{:?}", token);
        if token == tokens::Token::Eof {
            break;
        }
    }
}

#[test]
fn test_4() {
    let prog = "
    int main() {
        return -1 != -2;
    }
    ";
    let mut lexer = Lexer::new(prog.as_bytes());
    loop {
        let token = lexer.get_one_token();
        println!("{:?}", token);
        if token == tokens::Token::Eof {
            break;
        }
    }
}
