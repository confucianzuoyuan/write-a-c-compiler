pub fn round_way_from_zero(n: i64, x: i64) -> i64 {
    if x % n == 0 {
        x
    } else if x < 0 {
        x - n - (x % n)
    } else {
        x + n - (x % n)
    }
}
