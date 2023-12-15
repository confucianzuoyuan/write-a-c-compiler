#[derive(Clone, Debug, PartialEq)]
pub enum T {
    Identifier(String),
    Constant(i64),
    KWInt,
    KWReturn,
    KWVoid,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semicolon,
}
