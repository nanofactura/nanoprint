//! The Printers tab: the printers the system reports, which of them web pages may use, and a test
//! page to check one.

use std::{sync::Arc, time::Instant};

use icu_normalizer::DecomposingNormalizerBorrowed;
use serde::Serialize;
use tauri::{AppHandle, State};

use crate::{
    agent::{self, PrinterCache},
    i18n::{self, Locale},
    printers,
    settings::SettingsStore,
    store::{Event, EventKind, Store},
    ui_error::UiError,
};

/// Characters per line on a 58 mm roll, the narrowest common one. Wider rolls fit them too.
const LINE_WIDTH: usize = 32;

/// ESC/POS commands of the test page.
const INITIALIZE: &[u8] = b"\x1b@";
const CENTER: &[u8] = b"\x1ba\x01";
const LEFT: &[u8] = b"\x1ba\x00";
const BIG_BOLD: &[u8] = b"\x1d!\x11\x1bE\x01";
const NORMAL: &[u8] = b"\x1d!\x00\x1bE\x00";
/// Feeds the paper past the cutter and cuts it partially. Printers without a cutter only feed.
const FEED_AND_CUT: &[u8] = b"\x1dVB\x04";

/// A printer as the Printers tab shows it. Mirrors `PrinterView` in `src/lib/agent.ts`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrinterView {
    name: String,
    driver: Option<String>,
    /// The port (Windows) or device the printer is connected to, when the system says.
    connection: Option<String>,
    default: bool,
    /// Whether web pages may see it and print to it.
    shared: bool,
    /// It makes files or sends faxes (see [`printers::is_virtual`]): not offered to web pages by
    /// default, and no test page.
    is_virtual: bool,
}

/// The system's printers, those that print on paper first. `refresh` lists them again instead
/// of reusing the recent list.
#[tauri::command]
pub async fn list_printers(
    cache: State<'_, Arc<PrinterCache>>,
    settings: State<'_, SettingsStore>,
    refresh: bool,
) -> Result<Vec<PrinterView>, UiError> {
    let printers = cache
        .list(refresh)
        .await
        .map_err(|error| UiError::with_detail("printers_unavailable", &error))?;
    let settings = settings.get();
    let mut views: Vec<_> = printers
        .into_iter()
        .map(|printer| PrinterView {
            shared: settings.shares(&printer),
            is_virtual: printers::is_virtual(&printer),
            name: printer.name,
            driver: printer.driver,
            connection: printer.connection,
            default: printer.default,
        })
        .collect();
    views.sort_by_key(|view| view.is_virtual);
    Ok(views)
}

/// Offers a printer to web pages, or keeps it from them. Takes effect with the next request: a
/// hidden printer is left out of the list pages get, and jobs sent to it are refused.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub fn set_printer_shared(
    settings: State<'_, SettingsStore>,
    name: String,
    shared: bool,
) -> Result<(), UiError> {
    settings
        .update(|settings| {
            settings.shared_printers.insert(name, shared);
        })
        .map_err(|error| UiError::with_detail("settings", &error))?;
    Ok(())
}

/// Prints a short ESC/POS test page, whether or not web pages may use the printer. Recorded in
/// the history without a site. Virtual printers are refused: they would save the bytes as a broken
/// file.
#[tauri::command]
pub async fn print_test_page(
    app: AppHandle,
    cache: State<'_, Arc<PrinterCache>>,
    store: State<'_, Arc<Store>>,
    settings: State<'_, SettingsStore>,
    name: String,
) -> Result<(), UiError> {
    let found = cache
        .find(&name)
        .await
        .map_err(|error| UiError::with_detail("printers_unavailable", &error))?
        .ok_or(UiError::new("printer_not_found"))?;
    if printers::is_virtual(&found) {
        return Err(UiError::new("virtual_printer"));
    }
    let locale = i18n::resolve(settings.get().language);
    let job_name = i18n::t(locale, "printers.testPage.jobName");
    let data = test_page(locale, &name, &app.package_info().version.to_string());
    let bytes = data.len();
    let started = Instant::now();
    let printed = {
        let (name, job_name) = (name.clone(), job_name.clone());
        tauri::async_runtime::spawn_blocking(move || printers::print_raw(&name, &job_name, &data))
            .await
    };
    let error = match printed {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(error.to_string()),
        Err(error) => Some(error.to_string()),
    };
    let event = Event {
        origin: None,
        printer: Some(&name),
        job_name: Some(&job_name),
        bytes: Some(bytes),
        duration: Some(started.elapsed()),
        error: error.as_deref(),
    };
    if let Some(error) = &error {
        agent::record(&store, EventKind::JobFailed, &event);
        return Err(UiError::with_detail("print_failed", error));
    }
    agent::record(&store, EventKind::JobPrinted, &event);
    Ok(())
}

fn test_page(locale: Locale, printer: &str, version: &str) -> Vec<u8> {
    let rule = "-".repeat(LINE_WIDTH);
    let mut page = Vec::new();
    page.extend_from_slice(INITIALIZE);
    page.extend_from_slice(CENTER);
    page.extend_from_slice(BIG_BOLD);
    push_text(&mut page, "NanoPrint");
    page.extend_from_slice(NORMAL);
    push_text(&mut page, &i18n::t(locale, "printers.testPage.title"));
    page.extend_from_slice(LEFT);
    push_text(&mut page, &rule);
    push_text(
        &mut page,
        &format!(
            "{}: {printer}",
            i18n::t(locale, "printers.testPage.printer")
        ),
    );
    push_text(
        &mut page,
        &format!(
            "{}: {version}",
            i18n::t(locale, "printers.testPage.version")
        ),
    );
    push_text(&mut page, &rule);
    push_text(&mut page, &i18n::t(locale, "printers.testPage.check"));
    page.extend_from_slice(b"\n\n");
    page.extend_from_slice(FEED_AND_CUT);
    page
}

/// Appends `text` as printable ASCII, wrapped to [`LINE_WIDTH`].
fn push_text(page: &mut Vec<u8>, text: &str) {
    for line in wrap(&ascii(text), LINE_WIDTH) {
        page.extend_from_slice(line.as_bytes());
        page.push(b'\n');
    }
}

/// Printable ASCII only: thermal printers differ in which code page they use for the rest, and a
/// control character (in a printer's name, say) must not reach the printer as a command. Accents
/// are dropped (`Página` prints as `Pagina`); other characters print as `?`.
fn ascii(text: &str) -> String {
    DecomposingNormalizerBorrowed::new_nfd()
        .normalize(text)
        .chars()
        .filter_map(|c| match c {
            ' '..='~' => Some(c),
            // Accents, which the decomposition splits from their letters.
            '\u{300}'..='\u{36f}' | '¿' | '¡' => None,
            _ => Some('?'),
        })
        .collect()
}

/// Splits ASCII `text` into lines of at most `width` characters, between words where possible.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for mut word in text.split(' ').filter(|word| !word.is_empty()) {
        while word.len() > width {
            if !line.is_empty() {
                lines.push(std::mem::take(&mut line));
            }
            let (head, rest) = word.split_at(width);
            lines.push(head.to_owned());
            word = rest;
        }
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_text_to_printable_ascii() {
        assert_eq!(ascii("¿Página de prueba? Ñandú"), "Pagina de prueba? Nandu");
        assert_eq!(ascii("POS\x1b@\n€"), "POS?@??");
    }

    #[test]
    fn wraps_between_words_and_splits_long_ones() {
        assert_eq!(wrap("one two three", 7), ["one two", "three"]);
        assert_eq!(wrap("abcdefghij xy", 4), ["abcd", "efgh", "ij", "xy"]);
        assert_eq!(wrap("  ", 4), Vec::<String>::new());
    }

    #[test]
    fn a_printer_name_cannot_add_commands_to_the_test_page() {
        let page = test_page(Locale::Es, "POS\x1bp\x00\x19\x19", "1.2.3");
        let text = &page[INITIALIZE.len()..page.len() - FEED_AND_CUT.len()];
        let commands = [CENTER, LEFT, BIG_BOLD, NORMAL];
        let mut rest = text.to_vec();
        for command in commands {
            if let Some(at) = rest
                .windows(command.len())
                .position(|window| window == command)
            {
                rest.drain(at..at + command.len());
            }
        }
        assert!(
            rest.iter()
                .all(|byte| *byte == b'\n' || (b' '..=b'~').contains(byte))
        );
        assert!(page.starts_with(INITIALIZE) && page.ends_with(FEED_AND_CUT));
        let text = String::from_utf8_lossy(&page);
        assert!(text.contains("Impresora: POS?p???"));
        assert!(text.contains("Pagina de prueba"));
    }
}
