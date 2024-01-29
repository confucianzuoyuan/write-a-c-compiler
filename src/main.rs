mod assembly;
mod ast;
mod codegen;
mod emit;
mod identifier_resolution;
mod instruction_fixup;
mod ir;
mod ir_gen;
mod label_loops;
mod lexer;
mod parser;
mod replace_pseudos;
mod rounding;
mod symbols;
mod tokens;
mod typecheck;
mod types;
mod unique_ids;

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
    typecheck::typecheck(validated_ast.clone());
    let ir = ir_gen::gen(validated_ast);
    println!("{:?}", ir);
    println!("{}", ir);
    let asm_ast = codegen::gen(ir);
    println!("================= asm_ast =====================\r\n");
    emit::emit(asm_ast.clone());
    println!("================= asm_ast =====================\r\n");
    let mut replacement_state = replace_pseudos::ReplacementState::new();
    let asm_ast1 = replacement_state.replace_pseudos(asm_ast);
    // let symbol_table = symbol_table.clone();
    println!("================= asm_ast1 =====================\r\n");
    emit::emit(asm_ast1.clone());
    println!("================= asm_ast1 =====================\r\n");
    // println!("symbol_table: {:?}", symbol_table);
    let asm_ast2 = instruction_fixup::fixup_program(asm_ast1);
    emit::emit(asm_ast2);
}
