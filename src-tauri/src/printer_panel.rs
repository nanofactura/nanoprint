//! The Printers tab: the printers the system reports and the network printers the user adds,
//! which of them web pages may use, and a test page to check one.

use std::{sync::Arc, time::Instant};

use icu_normalizer::DecomposingNormalizerBorrowed;
use serde::Serialize;
use tauri::{AppHandle, State};

use crate::{
    agent::{self, Destination, PrinterCache},
    i18n::{self, Locale},
    printers::{
        self, Limitation,
        network::{self, Links},
    },
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
    /// Why it cannot print raw tickets, if it cannot: not offered to web pages by default, and no
    /// test page.
    limitation: Option<Limitation>,
    /// Added by the user by address, rather than reported by the system: the tab can remove it.
    network: bool,
}

/// The system's printers, those that can print tickets first. `refresh` lists them again instead
/// of reusing the recent list.
#[tauri::command]
pub async fn list_printers(
    cache: State<'_, Arc<PrinterCache>>,
    settings: State<'_, SettingsStore>,
    refresh: bool,
) -> Result<Vec<PrinterView>, UiError> {
    let unavailable = |error: String| UiError::with_detail("printers_unavailable", &error);
    if refresh {
        cache.list(true).await.map_err(unavailable)?;
    }
    let settings = settings.get();
    let mut views: Vec<_> = agent::all_printers(&cache, &settings)
        .await
        .map_err(unavailable)?
        .into_iter()
        .map(|destination| {
            let network = matches!(destination, Destination::Network(_));
            let printer = destination.printer();
            PrinterView {
                shared: settings.shares(&printer),
                limitation: printers::limitation(&printer),
                network,
                name: printer.name,
                driver: printer.driver,
                connection: printer.connection,
                default: printer.default,
            }
        })
        .collect();
    views.sort_by_key(|view| view.limitation.is_some());
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

/// Adds a network printer by address. Offered to web pages like any printer that prints tickets.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub async fn add_network_printer(
    cache: State<'_, Arc<PrinterCache>>,
    settings: State<'_, SettingsStore>,
    name: String,
    host: String,
    port: u16,
) -> Result<(), UiError> {
    let printer =
        network::check(&name, &host, port).map_err(|invalid| UiError::new(invalid.code()))?;
    // Names differing only in case would read as the same printer in the tab and in pages.
    let same_name = |other: &str| other.to_lowercase() == printer.name.to_lowercase();
    let system = cache
        .list(false)
        .await
        .map_err(|error| UiError::with_detail("printers_unavailable", &error))?;
    if system.iter().any(|known| same_name(&known.name))
        || settings
            .get()
            .network_printers
            .iter()
            .any(|known| same_name(&known.name))
    {
        return Err(UiError::new("printer_name_taken"));
    }
    log::info!("added the network printer at {}", printer.address());
    settings
        .update(|settings| {
            // A choice left by a removed printer with the same name does not carry over.
            settings.shared_printers.remove(&printer.name);
            settings.network_printers.push(printer);
        })
        .map_err(|error| UiError::with_detail("settings", &error))?;
    Ok(())
}

/// Removes a network printer the user added. Jobs to it are refused from then on.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub fn remove_network_printer(
    settings: State<'_, SettingsStore>,
    name: String,
) -> Result<(), UiError> {
    settings
        .update(|settings| {
            settings
                .network_printers
                .retain(|printer| printer.name != name);
            settings.shared_printers.remove(&name);
        })
        .map_err(|error| UiError::with_detail("settings", &error))?;
    Ok(())
}

/// Prints a short ESC/POS test page, whether or not web pages may use the printer. Recorded in
/// the history without a site. Printers that cannot print raw tickets are refused.
#[tauri::command]
pub async fn print_test_page(
    app: AppHandle,
    cache: State<'_, Arc<PrinterCache>>,
    links: State<'_, Arc<Links>>,
    store: State<'_, Arc<Store>>,
    settings: State<'_, SettingsStore>,
    name: String,
) -> Result<(), UiError> {
    let settings = settings.get();
    let found = agent::find_printer(&cache, &settings, &name)
        .await
        .map_err(|error| UiError::with_detail("printers_unavailable", &error))?
        .ok_or(UiError::new("printer_not_found"))?;
    match printers::limitation(&found.printer()) {
        Some(Limitation::Virtual) => return Err(UiError::new("virtual_printer")),
        Some(Limitation::EasyPrint) => return Err(UiError::new("easy_print_printer")),
        None => {}
    }
    let locale = i18n::resolve(settings.language);
    let job_name = i18n::t(locale, "printers.testPage.jobName");
    let data = test_page(locale, &name, &app.package_info().version.to_string());
    let bytes = data.len();
    let started = Instant::now();
    let error = agent::send(&links, found, job_name.clone(), data)
        .await
        .err();
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
