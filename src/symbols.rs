use crate::types;
use lazy_static::lazy_static;
use std::{collections::HashMap, sync::Mutex};

lazy_static! {
    static ref SYMBOL_TABLE: Mutex<HashMap<String, Entry>> = Mutex::new(HashMap::new());
}

#[derive(Clone, Debug, PartialEq)]
pub enum InitialValue {
    Tentative,
    Initial(i64),
    NoInitializer,
}

#[derive(Clone, Debug, PartialEq)]
pub enum IdentifierAttrs {
    FunAttr {
        defined: bool,
        global: bool,
        stack_frame_size: i64,
    },
    StaticAttr {
        init: InitialValue,
        global: bool,
    },
    LocalAttr,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub t: types::Type,
    pub attrs: IdentifierAttrs,
}

pub fn add_automatic_var(name: String, t: types::Type) {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    let entry = Entry {
        t: t,
        attrs: IdentifierAttrs::LocalAttr,
    };
    _map.insert(name, entry);
}

pub fn add_static_var(name: String, t: types::Type, global: bool, init: InitialValue) {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    let entry = Entry {
        t: t,
        attrs: IdentifierAttrs::StaticAttr {
            init: init,
            global: global,
        },
    };
    _map.insert(name, entry);
}

pub fn add_fun(name: String, t: types::Type, global: bool, defined: bool) {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    let entry = Entry {
        t: t,
        attrs: IdentifierAttrs::FunAttr {
            defined: defined,
            global: global,
            stack_frame_size: 0,
        },
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

pub fn is_global(name: String) -> bool {
    match get(name).attrs {
        IdentifierAttrs::LocalAttr => false,
        IdentifierAttrs::StaticAttr { init: _, global } => global,
        IdentifierAttrs::FunAttr {
            defined: _,
            global,
            stack_frame_size: _,
        } => global,
    }
}

pub fn is_static(name: String) -> bool {
    match get_opt(name) {
        Some(entry) => match entry.attrs {
            IdentifierAttrs::LocalAttr => false,
            IdentifierAttrs::StaticAttr { init: _, global: _ } => true,
            IdentifierAttrs::FunAttr {
                defined: _,
                global: _,
                stack_frame_size: _,
            } => panic!("内部错误：函数没有storage duration。"),
        },
        None => false,
    }
}

pub fn is_defined(name: String) -> bool {
    let _map = SYMBOL_TABLE.lock().unwrap();
    _map.contains_key(&name)
}

pub fn bindings() -> Vec<(String, Entry)> {
    let _map = SYMBOL_TABLE.lock().unwrap();
    let mut bindings = vec![];
    for key in _map.keys() {
        bindings.push(((*key).clone(), (*_map.get(key).unwrap()).clone()));
    }
    bindings
}

pub fn set_bytes_required(name: String, bytes_required: i64) {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    let entry = _map.get(&name).unwrap();
    let new_entry = match entry.attrs {
        IdentifierAttrs::FunAttr {
            defined,
            global,
            stack_frame_size: _,
        } => Entry {
            t: entry.t.clone(),
            attrs: IdentifierAttrs::FunAttr {
                defined: defined,
                global: global,
                stack_frame_size: bytes_required,
            },
        },
        _ => panic!(),
    };
    _map.insert(name, new_entry);
}

pub fn get_bytes_required(name: String) -> i64 {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    match _map.get(&name) {
        Some(Entry {
            t: _,
            attrs:
                IdentifierAttrs::FunAttr {
                    defined: _,
                    global: _,
                    stack_frame_size,
                },
        }) => *stack_frame_size,
        _ => panic!(),
    }
}

pub fn iter<F>(f: F) where F: Fn(String, Entry) -> () {
    let mut _map = SYMBOL_TABLE.lock().unwrap();
    for (k, v) in _map.iter() {
        f(k, v);
    }
}