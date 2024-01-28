mod lexer;
mod tokens;
mod parser;
mod ast;
mod unique_ids;
mod ir;
mod ir_gen;
mod assembly;
mod codegen;
mod instruction_fixup;
mod replace_pseudos;
mod identifier_resolution;
mod label_loops;
mod typecheck;
mod symbols;
mod types;
mod emit;
mod rounding;

fn main() {
    let program = "
    int add(int a, int b) {
        return a + b;
    }
    
    int main(void) {
        int sum = add(1 + 2, 4);
        return sum + sum;
    }
    ";
    let mut lexer = lexer::Lexer::new(program.as_bytes());
    let tokens = lexer.lex();
    println!("{:?}", tokens);
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse();
    println!("{:?}", ast);
    let resolved_ast = identifier_resolution::resolve(ast.clone());
    println!("resolved_ast: {:?}", resolved_ast);
    let validated_ast = label_loops::label_loops(resolved_ast);
    println!("validated_ast: {:?}", validated_ast);
    let mut type_check = typecheck::TypeCheck::new();
    type_check.typecheck(validated_ast.clone());
    let ir = ir_gen::gen(validated_ast);
    println!("{:?}", ir);
    println!("{}", ir);
    let asm_ast = codegen::gen(ir);
    emit::emit(asm_ast.clone());
    let mut replacement_state = replace_pseudos::ReplacementState::new();
    let asm_ast1 = replacement_state.replace_pseudos(asm_ast, type_check.symbol_table.clone());
    println!("asm_ast1: {:?}", asm_ast1);
    let asm_ast2 = instruction_fixup::fixup_program( asm_ast1, type_check.symbol_table);
    println!("asm_ast2: {:?}", asm_ast2);
    emit::emit(asm_ast2);
}