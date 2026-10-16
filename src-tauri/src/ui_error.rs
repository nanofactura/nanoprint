//! Errors as the windows show them: a code each window translates (`error.code.<code>` in
//! `locales/`), plus the system's own message when there is one. NanoPrint's own wording never
//! reaches the user untranslated.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UiError {
    code: &'static str,
    /// What the operating system or a tool said: already in the user's language, or technical.
    detail: Option<String>,
}

impl UiError {
    pub const fn new(code: &'static str) -> Self {
        Self { code, detail: None }
    }

    pub fn with_detail(code: &'static str, detail: &dyn std::fmt::Display) -> Self {
        Self {
            code,
            detail: Some(detail.to_string()),
        }
    }
}
