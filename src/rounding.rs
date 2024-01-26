pub fn round_way_from_zero(n: i64, x: i64) -> i64 {
    match x {
        _x if _x % n == 0 => _x,
        _x if _x < 0 => _x - n - (_x % n),
        _x => _x + n - (_x % n),
    }
}
