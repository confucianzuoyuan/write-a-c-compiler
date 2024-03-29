use std::collections::HashMap;

use crate::{assembly, symbols};

#[derive(Clone, Debug, PartialEq)]
pub struct ReplacementState {
    pub current_offset: i64,
    offset_map: HashMap<String, i64>,
}

impl ReplacementState {
    pub fn new() -> Self {
        ReplacementState {
            current_offset: 0,
            offset_map: HashMap::new(),
        }
    }

    fn replace_operand(&mut self, operand: assembly::Operand) -> assembly::Operand {
        match operand {
            assembly::Operand::Pseudo(s) => {
                if symbols::is_static(s.clone()) {
                    assembly::Operand::Data(s)
                } else {
                    if let Some(offset) = self.offset_map.get(&s) {
                        assembly::Operand::Stack(*offset)
                    } else {
                        self.current_offset = self.current_offset - 4;
                        self.offset_map.insert(s, self.current_offset);
                        assembly::Operand::Stack(self.current_offset)
                    }
                }
            }
            other => other,
        }
    }

    fn replace_pseudos_in_instruction(
        &mut self,
        instruction: assembly::Instruction,
    ) -> assembly::Instruction {
        match instruction {
            assembly::Instruction::Mov(t, src, dst) => {
                let new_src = self.replace_operand(src);
                let new_dst = self.replace_operand(dst);
                assembly::Instruction::Mov(t, new_src, new_dst)
            }
            assembly::Instruction::Unary(t, op, dst) => {
                let new_dst = self.replace_operand(dst);
                assembly::Instruction::Unary(t, op, new_dst)
            }
            assembly::Instruction::Binary { op, t, src, dst } => {
                let new_src = self.replace_operand(src);
                let new_dst = self.replace_operand(dst);
                assembly::Instruction::Binary {
                    op: op,
                    t: t,
                    src: new_src,
                    dst: new_dst,
                }
            }
            assembly::Instruction::Cmp(t, op1, op2) => {
                let new_op1 = self.replace_operand(op1);
                let new_op2 = self.replace_operand(op2);
                assembly::Instruction::Cmp(t, new_op1, new_op2)
            }
            assembly::Instruction::Idiv(t, op) => {
                let new_op = self.replace_operand(op);
                assembly::Instruction::Idiv(t, new_op)
            }
            assembly::Instruction::SetCC(code, op) => {
                let new_op = self.replace_operand(op);
                assembly::Instruction::SetCC(code, new_op)
            }
            assembly::Instruction::Push(op) => {
                let new_op = self.replace_operand(op);
                assembly::Instruction::Push(new_op)
            }
            other @ (assembly::Instruction::Ret
            | assembly::Instruction::Cdq(_)
            | assembly::Instruction::Label(_)
            | assembly::Instruction::JmpCC(_, _)
            | assembly::Instruction::Jmp(_)
            | assembly::Instruction::DeallocateStack(_)
            | assembly::Instruction::Call(_)
            | assembly::Instruction::AllocateStack(_)) => other,
        }
    }

    fn replace_pseudos_in_tl(&mut self, f: assembly::TopLevel) -> assembly::TopLevel {
        match f {
            assembly::TopLevel::Function {
                name,
                global,
                instructions,
            } => {
                self.current_offset = 0;
                self.offset_map = HashMap::new();
                let mut fixup_instructions = vec![];
                for i in instructions {
                    fixup_instructions.push(self.replace_pseudos_in_instruction(i));
                }
                symbols::set_bytes_required(name.clone(), self.current_offset);
                assembly::TopLevel::Function {
                    name: name,
                    global: global,
                    instructions: fixup_instructions,
                }
            }
            static_var => static_var,
        }
    }

    pub fn replace_pseudos(&mut self, program: assembly::T) -> assembly::T {
        match program {
            assembly::T::Program(tls) => {
                let mut fixed_defs = vec![];
                for tl in tls {
                    fixed_defs.push(self.replace_pseudos_in_tl(tl));
                }
                assembly::T::Program(fixed_defs)
            }
        }
    }
}
