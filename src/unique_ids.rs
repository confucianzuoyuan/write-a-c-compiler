static mut COUNTER: usize = 0;

pub fn make_temporary() -> String {
    unsafe {
        let n = COUNTER;
        COUNTER += 1;
        format!("tmp.{n}")
    }
}

pub fn make_label(prefix: String) -> String {
    unsafe {
        let n = COUNTER;
        COUNTER += 1;
        format!("{prefix}.{n}")
    }
}