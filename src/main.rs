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
mod var_resolution;
mod label_loops;

fn main() {
    let program = "
    int main(void) {
        int a = 1;
        do {
            a = a * 2;
        } while(a < 11);
    
        return a;
    }
    ";
    let mut lexer = lexer::Lexer::new(program.as_bytes());
    let tokens = lexer.lex();
    println!("{:?}", tokens);
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse();
    println!("{:?}", ast);
    let resolved_ast = var_resolution::resolve(ast.clone());
    println!("resolved_ast: {:?}", resolved_ast);
    let validated_ast = label_loops::label_loops(resolved_ast);
    println!("validated_ast: {:?}", validated_ast);
    let ir = ir_gen::gen(validated_ast);
    println!("{}", ir);
    let asm_ast = codegen::gen(ir);
    println!("{}", asm_ast);
    let mut replacement_state = replace_pseudos::ReplacementState::new();
    let asm_ast1 = replacement_state.replace_pseudos(asm_ast);
    println!("{}", asm_ast1);
    let asm_ast2 = instruction_fixup::fixup_program(replacement_state.current_offset, asm_ast1);
    println!("{}", asm_ast2);
}