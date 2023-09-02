use std::sync::OnceLock;

use regex::Regex;

static REMOVAL_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn sanitize_filename(input: &str) -> String {
    REMOVAL_REGEX
        .get_or_init(|| Regex::new(r#"[<>:"/\|?*']"#).unwrap())
        .replace_all(&input.replace(" - ", "-").replace(" ", "_"), "")
        .to_string()
}
