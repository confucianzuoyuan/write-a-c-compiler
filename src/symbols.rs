use crate::types;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub t: types::Type,
    pub is_defined: bool,
    pub stack_frame_size: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SymbolTable {
    pub symbol_table: HashMap<String, Entry>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbol_table: HashMap::new(),
        }
    }

    pub fn add_var(&mut self, name: String, t: types::Type) {
        self.symbol_table.insert(
            name,
            Entry {
                t: t,
                is_defined: false,
                stack_frame_size: 0,
            },
        );
    }

    pub fn add_fun(&mut self, name: String, t: types::Type, is_defined: bool) {
        self.symbol_table.insert(
            name,
            Entry {
                t: t,
                is_defined: is_defined,
                stack_frame_size: 0,
            },
        );
    }

    pub fn get(&self, name: String) -> &Entry {
        self.symbol_table.get(&name).unwrap()
    }

    pub fn get_opt(&self, name: String) -> Option<&Entry> {
        self.symbol_table.get(&name)
    }

    pub fn is_defined(&self, name: String) -> bool {
        self.symbol_table.contains_key(&name)
    }

    pub fn set_bytes_required(&mut self, name: String, bytes_required: i64) {
        let entry = self.symbol_table.get(&name).unwrap();
        let new_entry = Entry {
            t: entry.t.clone(),
            is_defined: entry.is_defined.clone(),
            stack_frame_size: bytes_required,
        };
        self.symbol_table.insert(name, new_entry);
    }
}
