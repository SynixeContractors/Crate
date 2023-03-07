#[must_use]
/// Format a number with commas as used in American currency.
pub fn money(i: i32, pad_negative: bool) -> String {
    let i_str = i.abs().to_string();
    let mut s = String::with_capacity(i_str.len() + 2);
    let a = i_str.chars().rev().enumerate();
    for (idx, val) in a {
        if idx != 0 && idx % 3 == 0 {
            s.push(',');
        }
        s.push(val);
    }
    s.push('$');
    if i < 0 {
        s.push('-');
    } else if pad_negative {
        s.push(' ');
    }
    s.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money() {
        assert_eq!(money(0, false), "$0");
        assert_eq!(money(1, false), "$1");
        assert_eq!(money(10, false), "$10");
        assert_eq!(money(100, false), "$100");
        assert_eq!(money(1000, false), "$1,000");
        assert_eq!(money(10000, false), "$10,000");
        assert_eq!(money(100_000, false), "$100,000");
        assert_eq!(money(1_000_000, false), "$1,000,000");
        assert_eq!(money(-1, false), "-$1");
        assert_eq!(money(-10, false), "-$10");
        assert_eq!(money(-100, false), "-$100");
        assert_eq!(money(-1000, false), "-$1,000");
        assert_eq!(money(-10000, false), "-$10,000");
        assert_eq!(money(-100_000, false), "-$100,000");
        assert_eq!(money(-1_000_000, false), "-$1,000,000");
        assert_eq!(money(0, true), " $0");
        assert_eq!(money(1, true), " $1");
        assert_eq!(money(10, true), " $10");
    }
}
