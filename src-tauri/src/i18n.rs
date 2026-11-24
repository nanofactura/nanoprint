//! Strings for the natively rendered UI (tray menu, notifications). They share
//! `locales/*.json` with the web UI so every text is translated in one place.

use std::{collections::HashMap, sync::LazyLock};

use serde::Serialize;

use crate::settings::LanguagePreference;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    En,
    Es,
    #[serde(rename = "pt-BR")]
    PtBr,
}

type Catalog = HashMap<String, String>;

static EN: LazyLock<Catalog> = LazyLock::new(|| parse(include_str!("../../locales/en.json")));
static ES: LazyLock<Catalog> = LazyLock::new(|| parse(include_str!("../../locales/es.json")));
static PT_BR: LazyLock<Catalog> = LazyLock::new(|| parse(include_str!("../../locales/pt-BR.json")));

fn parse(json: &str) -> Catalog {
    // The catalogs are embedded at compile time and covered by tests: invalid JSON is a build
    // defect, not a runtime condition.
    serde_json::from_str(json).expect("embedded locale catalog must be valid JSON")
}

/// Detects the operating system language, falling back to English.
pub fn system_locale() -> Locale {
    sys_locale::get_locale().map_or(Locale::En, |tag| from_tag(&tag))
}

/// Maps a BCP 47 or POSIX tag by language, using Brazilian Portuguese for all Portuguese regions.
fn from_tag(tag: &str) -> Locale {
    let language = tag.split(['-', '_', '.']).next().unwrap_or_default();
    if language.eq_ignore_ascii_case("es") {
        Locale::Es
    } else if language.eq_ignore_ascii_case("pt") {
        Locale::PtBr
    } else {
        Locale::En
    }
}

pub fn resolve(preference: LanguagePreference) -> Locale {
    match preference {
        LanguagePreference::System => system_locale(),
        LanguagePreference::En => Locale::En,
        LanguagePreference::Es => Locale::Es,
        LanguagePreference::PtBr => Locale::PtBr,
    }
}

/// Returns the translation of `key`, falling back to English and then to the key itself.
pub fn t(locale: Locale, key: &str) -> String {
    let catalog = match locale {
        Locale::En => &*EN,
        Locale::Es => &*ES,
        Locale::PtBr => &*PT_BR,
    };
    catalog
        .get(key)
        .or_else(|| EN.get(key))
        .cloned()
        .unwrap_or_else(|| key.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogs_define_the_same_keys() {
        let mut en: Vec<_> = EN.keys().collect();
        en.sort();
        for catalog in [&*ES, &*PT_BR] {
            let mut keys: Vec<_> = catalog.keys().collect();
            keys.sort();
            assert_eq!(en, keys);
        }
    }

    #[test]
    fn maps_language_tags() {
        assert_eq!(from_tag("es-AR"), Locale::Es);
        assert_eq!(from_tag("es_ES.UTF-8"), Locale::Es);
        assert_eq!(from_tag("ES"), Locale::Es);
        assert_eq!(from_tag("en-US"), Locale::En);
        assert_eq!(from_tag("pt-BR"), Locale::PtBr);
        assert_eq!(from_tag("pt-PT"), Locale::PtBr);
        assert_eq!(from_tag("pt_BR.UTF-8"), Locale::PtBr);
        assert_eq!(from_tag("PT"), Locale::PtBr);
        assert_eq!(from_tag("eu-ES"), Locale::En);
        assert_eq!(from_tag(""), Locale::En);
    }

    #[test]
    fn resolves_and_serializes_brazilian_portuguese() {
        let tag = r#""pt-BR""#;
        let preference: LanguagePreference = serde_json::from_str(tag).unwrap();
        assert_eq!(preference, LanguagePreference::PtBr);
        assert_eq!(serde_json::to_string(&preference).unwrap(), tag);
        let locale = resolve(preference);
        assert_eq!(serde_json::to_string(&locale).unwrap(), tag);
        assert_eq!(t(locale, "tray.open"), "Abrir painel");
    }

    #[test]
    fn translates_with_fallback_to_the_key() {
        assert_eq!(t(Locale::Es, "tray.open"), "Abrir panel");
        assert_eq!(t(Locale::Es, "missing.key"), "missing.key");
    }
}
