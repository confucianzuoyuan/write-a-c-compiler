use crate::assembly;
use lazy_static::lazy_static;
use std::{collections::HashMap, sync::Mutex};

#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    Fun {
        defined: bool,
        bytes_required: i64,
    },
    Obj {
        t: assembly::AsmType,
        is_static: bool,
    },
}

lazy_static! {
    static ref SYMBOL_TABLE: Mutex<HashMap<String, Entry>> = Mutex::new(HashMap::new());
}

pub fn add_fun(fun_name: String, defined: bool) {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    let entry = Entry::Fun {
        defined: defined,
        bytes_required: 0,
    };
    _map.insert(fun_name, entry);
}

pub fn add_var(var_name: String, t: assembly::AsmType, is_static: bool) {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    let entry = Entry::Obj {
        t: t,
        is_static: is_static,
    };
    _map.insert(var_name, entry);
}

pub fn get_bytes_required(fun_name: String) -> i64 {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    match _map.get(&fun_name).unwrap() {
        Entry::Fun {
            defined: _,
            bytes_required,
        } => *bytes_required,
        Entry::Obj { t: _, is_static: _ } => panic!("内部错误：不是一个函数。"),
    }
}

pub fn get_size(var_name: String) -> i64 {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    match _map.get(&var_name).unwrap() {
        Entry::Obj {
            t: assembly::AsmType::Longword,
            is_static: _,
        } => 4,
        Entry::Obj {
            t: assembly::AsmType::Quadword,
            is_static: _,
        } => 8,
        Entry::Fun {
            defined: _,
            bytes_required: _,
        } => panic!("内部错误：这是一个函数，不是一个对象。"),
    }
}

pub fn get_alignment(var_name: String) -> i64 {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    match _map.get(&var_name).unwrap() {
        Entry::Obj {
            t: assembly::AsmType::Longword,
            is_static: _,
        } => 4,
        Entry::Obj {
            t: assembly::AsmType::Quadword,
            is_static: _,
        } => 8,
        Entry::Fun {
            defined: _,
            bytes_required: _,
        } => panic!("内部错误：这是一个函数，不是一个对象。"),
    }
}

pub fn is_defined(fun_name: String) -> bool {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    match _map.get(&fun_name).unwrap() {
        Entry::Fun {
            defined,
            bytes_required: _,
        } => *defined,
        _ => panic!("内部错误：不是函数。"),
    }
}

pub fn is_static(var_name: String) -> bool {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    match _map.get(&var_name).unwrap() {
        Entry::Obj { t: _, is_static } => *is_static,
        Entry::Fun {
            defined: _,
            bytes_required: _,
        } => panic!("内部错误：函数没有storage duration。"),
    }
}
