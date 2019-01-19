pub fn trans(input: &str) -> u32 {
    if let Ok(result) = input.parse() {
        return result;
    }

    if let Ok(x) = input.to_lowercase().parse::<humantime::Duration>() {
        return x.as_secs() as u32;
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trans() {
        assert_eq!(trans("15"), 15);
        assert_eq!(trans("15s"), 15);
        assert_eq!(trans("15S"), 15);
        assert_eq!(trans("5m"), 300);
        assert_eq!(trans("2h"), 7200);
        assert_eq!(trans("1d"), 86400);
    }
}
