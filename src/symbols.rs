use crate::types;
use lazy_static::lazy_static;
use std::{collections::HashMap, sync::Mutex};

lazy_static! {
    static ref SYMBOL_TABLE: Mutex<HashMap<String, Entry>> = Mutex::new(HashMap::new());
}

pub fn add_var(name: String, t: types::Type) {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    let entry = Entry {
        t: t,
        is_defined: false,
        stack_frame_size: 0,
    };
    _map.insert(name, entry);
}

pub fn add_fun(name: String, t: types::Type, is_defined: bool) {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    let entry = Entry {
        t: t,
        is_defined: is_defined,
        stack_frame_size: 0,
    };
    _map.insert(name, entry);
}

pub fn get(name: String) -> Entry {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    _map.get(&name).unwrap().clone()
}

pub fn get_opt(name: String) -> Option<Entry> {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    if _map.get(&name).is_some() {
        Some(_map.get(&name).unwrap().clone())
    } else {
        None
    }
}

pub fn set_bytes_required(name: String, bytes_required: i64) {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    let entry = _map.get(&name).unwrap();
    let new_entry = Entry {
        t: entry.t.clone(),
        is_defined: entry.is_defined.clone(),
        stack_frame_size: bytes_required,
    };
    _map.insert(name, new_entry);
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub t: types::Type,
    pub is_defined: bool,
    pub stack_frame_size: i64,
}
