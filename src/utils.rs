pub fn is_unix_symbol(s: &str) -> bool {
    const PROTECTED_SYMBOL: [&str; 3] = ["/", ".", ".."];
    PROTECTED_SYMBOL.contains(&s)
}