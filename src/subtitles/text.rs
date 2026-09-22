pub fn is_punctuation_or_symbol(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_punctuation() || "。，！？；：「」『』《》、…—।॥".contains(c))
}

pub fn is_cjk(c: char) -> bool {
    let u = c as u32;
    (0x4E00..=0x9FFF).contains(&u)
        || (0x3400..=0x4DBF).contains(&u)
        || (0x3040..=0x309F).contains(&u)
        || (0x30A0..=0x30FF).contains(&u)
        || (0xAC00..=0xD7AF).contains(&u)
}
