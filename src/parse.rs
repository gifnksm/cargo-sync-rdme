pub(crate) fn is_valid_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if is_ident_start(c) => (),
        _ => return false,
    }
    for c in chars {
        if !is_ident_continue(c) {
            return false;
        }
    }
    true
}

pub(crate) fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic()
}

pub(crate) fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}
