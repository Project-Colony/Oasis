//! The console messages and notifications Oasis shows, in English and French.
//!
//! English is canonical: a key is named after its English string, and a new
//! string is written in `en.rs` first. The active locale is the one in
//! `colony_ui::i18n`, so Oasis and the shared Colony labels always agree.
//!
//! Error chains from `anyhow` (the `.context()` strings) stay in English: they
//! go to stderr for bug reports, never into a notification.

mod en;
mod fr;

use crate::config::Language;
use colony_ui::i18n::{Locale, locale, set_locale};

/// Sets the locale every `t` call reads. `Auto` follows the system language.
pub fn set_language(language: Language) {
    let locale = match language {
        Language::En => Locale::En,
        Language::Fr => Locale::Fr,
        Language::Auto => Locale::from_tag(&system_language()),
    };
    set_locale(locale);
}

/// The OS language: the user's UI language on Windows and macOS, `LANGUAGE`,
/// `LC_ALL`, `LC_MESSAGES` or `LANG` elsewhere. Empty when nothing is set,
/// which gives English.
fn system_language() -> String {
    sys_locale::get_locale()
        .or_else(|| {
            ["LC_ALL", "LC_MESSAGES", "LANG"]
                .into_iter()
                .find_map(|var| std::env::var(var).ok().filter(|value| !value.is_empty()))
        })
        .unwrap_or_default()
}

/// The string for `key` in the active locale.
pub fn t(key: &'static str) -> &'static str {
    t_in(locale(), key)
}

/// `t`, with each `{name}` placeholder replaced by its value.
pub fn t_fmt(key: &'static str, vars: &[(&str, &str)]) -> String {
    vars.iter().fold(t(key).to_string(), |text, (name, value)| {
        text.replace(&format!("{{{name}}}"), value)
    })
}

/// The string for `key` in `locale`, then in English, then the key itself, so a
/// missing string shows up as its key instead of a blank line.
pub fn t_in(locale: Locale, key: &'static str) -> &'static str {
    let table = match locale {
        Locale::En => en::STRINGS,
        Locale::Fr => fr::STRINGS,
    };
    find(table, key)
        .or_else(|| find(en::STRINGS, key))
        .unwrap_or(key)
}

fn find(table: &[(&str, &'static str)], key: &str) -> Option<&'static str> {
    table.iter().find(|(k, _)| *k == key).map(|&(_, text)| text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn keys(table: &[(&'static str, &str)]) -> BTreeSet<&'static str> {
        table.iter().map(|&(key, _)| key).collect()
    }

    /// The `{name}` placeholders in a string.
    fn placeholders(text: &str) -> BTreeSet<&str> {
        text.split('{')
            .skip(1)
            .filter_map(|rest| rest.split_once('}').map(|(name, _)| name))
            .collect()
    }

    #[test]
    fn fr_and_en_have_identical_key_sets() {
        let (en, fr) = (keys(en::STRINGS), keys(fr::STRINGS));
        assert_eq!(en.len(), en::STRINGS.len(), "a key appears twice in en.rs");
        assert_eq!(fr.len(), fr::STRINGS.len(), "a key appears twice in fr.rs");
        let only_en: Vec<_> = en.difference(&fr).collect();
        let only_fr: Vec<_> = fr.difference(&en).collect();
        assert!(
            only_en.is_empty() && only_fr.is_empty(),
            "only in en: {only_en:?}; only in fr: {only_fr:?}"
        );
    }

    #[test]
    fn fr_strings_keep_the_english_placeholders() {
        for &(key, english) in en::STRINGS {
            assert_eq!(
                placeholders(t_in(Locale::Fr, key)),
                placeholders(english),
                "placeholders of {key}"
            );
        }
    }

    /// Every `src/*.rs` file, read at test time so a new module is covered
    /// without a list to keep up to date.
    fn sources() -> Vec<String> {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
        std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
            .map(|path| std::fs::read_to_string(path).unwrap())
            .collect()
    }

    /// The keys passed as literals to `t` or `t_fmt`, also when rustfmt puts
    /// the key on the next line.
    fn used_keys(source: &str) -> Vec<&str> {
        let mut used = Vec::new();
        for call in ["t(", "t_fmt("] {
            for (at, _) in source.match_indices(call) {
                // `print(` or `assert(` end in `t(` too.
                let prefix = source[..at].chars().next_back();
                if prefix.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                    continue;
                }
                let Some(rest) = source[at + call.len()..].trim_start().strip_prefix('"') else {
                    continue;
                };
                used.extend(rest.split('"').next());
            }
        }
        used
    }

    #[test]
    fn used_keys_finds_literal_keys_only() {
        let source = "t(\"a\"); t_fmt(\n    \"b\",\n); print(\"c\"); t(key);";
        assert_eq!(used_keys(source), ["a", "b"]);
    }

    #[test]
    fn every_key_used_in_src_exists_in_en() {
        let sources = sources();
        let en = keys(en::STRINGS);
        let missing: Vec<_> = sources
            .iter()
            .flat_map(|source| used_keys(source))
            .filter(|key| !en.contains(key))
            .collect();
        assert!(missing.is_empty(), "keys missing from en.rs: {missing:?}");

        // The other way round: a key nothing looks up is a string nobody sees.
        let dead: Vec<_> = en
            .iter()
            .filter(|key| {
                let needle = format!("\"{key}\"");
                !sources.iter().any(|source| source.contains(&needle))
            })
            .collect();
        assert!(dead.is_empty(), "keys in en.rs that nothing uses: {dead:?}");
    }

    /// Tests never set the locale, so `t` reads the English default.
    #[test]
    fn t_fmt_fills_the_placeholders() {
        assert_eq!(
            t_fmt("current_weather", &[("city", "Lyon")]),
            "Current weather - Lyon"
        );
        assert_eq!(
            t_fmt("active_hotkeys", &[("primary", "A"), ("secondary", "B")]),
            "Active hotkeys: A and B"
        );
    }

    #[test]
    fn missing_key_falls_back_to_the_key() {
        assert_eq!(t_in(Locale::Fr, "no_such_key"), "no_such_key");
    }
}
