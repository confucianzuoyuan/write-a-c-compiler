mod assembly;
mod ast;
mod codegen;
mod tokens;
mod lex;
mod parse;
mod emit;

/// riscv64-unknown-elf-gcc tmp.s
/// qemu-riscv64 tmp
/// echo $?
/// 20
fn main() {
    let program = "
        int main(void) {
            return 20;
        }
    ";
    let mut tokens = lex::lex(program);
    let ast = parse::parse(&mut tokens);
    let code = codegen::gen(ast);
    emit::emit(code);
}
