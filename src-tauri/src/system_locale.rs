// LC_ALL overrides the category-specific locale, which overrides LANG.
fn select_locale(mut read: impl FnMut(&str) -> Option<String>) -> Option<String> {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .filter_map(|key| read(key))
        .find(|value| !value.trim().is_empty())
}

#[tauri::command]
pub fn system_locale() -> Option<String> {
    select_locale(|key| std::env::var(key).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_environment_precedence() {
        for (all, messages, lang, expected) in [
            (
                Some("en_US.UTF-8"),
                Some("zh_CN.UTF-8"),
                Some("ja_JP.UTF-8"),
                Some("en_US.UTF-8"),
            ),
            (
                Some(""),
                Some("zh_CN.UTF-8"),
                Some("en_US.UTF-8"),
                Some("zh_CN.UTF-8"),
            ),
            (None, Some(" "), Some("C.UTF-8"), Some("C.UTF-8")),
            (None, None, None, None),
        ] {
            let result = select_locale(|key| {
                match key {
                    "LC_ALL" => all,
                    "LC_MESSAGES" => messages,
                    "LANG" => lang,
                    _ => None,
                }
                .map(str::to_owned)
            });
            assert_eq!(result.as_deref(), expected);
        }
    }
}
