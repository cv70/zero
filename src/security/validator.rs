use regex::Regex;

lazy_static::lazy_static! {
    static ref KEY_REGEX: Regex = Regex::new(r"^[A-Za-z0-9_\-:.]+$").unwrap();
}

pub fn is_valid_key(key: &str) -> bool {
    KEY_REGEX.is_match(key)
}
