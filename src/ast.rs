#[derive(Clone, Debug, PartialEq)]
pub enum Exp {
    Constant { value: i64 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Return { exp: Exp },
}

#[derive(Clone, Debug, PartialEq)]
pub enum FunctionDefinition {
    Function { name: String, body: Statement },
}

#[derive(Clone, Debug, PartialEq)]
pub enum T {
    Program {
        function_definition: FunctionDefinition,
    },
}
