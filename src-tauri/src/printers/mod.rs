//! Printers installed in the operating system, printed to in raw mode (the bytes reach the printer
//! untouched by the driver).

#[cfg(not(windows))]
mod cups;
#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
pub use cups::{list, print_raw};
#[cfg(windows)]
pub use windows::{list, print_raw};

use crate::server::Printer;

/// Windows ports of printers that save a file or send a fax instead of printing.
const VIRTUAL_PORTS: [&str; 5] = ["PORTPROMPT:", "FILE:", "NUL:", "SHRFAX:", "XPSPort:"];
/// Words in the name or driver of such printers, on any system: `Microsoft Print to PDF`,
/// `Microsoft XPS Document Writer`, `Fax`, `OneNote`, `MS Publisher Color Printer`, CUPS-PDF…
const VIRTUAL_WORDS: [&str; 6] = [
    "pdf",
    "xps",
    "fax",
    "onenote",
    "ms publisher",
    "document writer",
];

/// Whether `printer` makes files or sends faxes rather than printing on paper. Raw ESC/POS bytes
/// are useless to these printers (a PDF writer saves them as a broken file), so web pages do not
/// get them unless the user offers them.
pub fn is_virtual(printer: &Printer) -> bool {
    let port = printer.connection.as_deref().unwrap_or_default();
    let described = format!(
        "{} {}",
        printer.name,
        printer.driver.as_deref().unwrap_or_default()
    )
    .to_lowercase();
    VIRTUAL_PORTS
        .iter()
        .any(|virtual_port| port.eq_ignore_ascii_case(virtual_port))
        || port.to_lowercase().starts_with("microsoft.office.onenote")
        || VIRTUAL_WORDS.iter().any(|word| described.contains(word))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{operation} failed: {source}")]
    Os {
        operation: &'static str,
        source: std::io::Error,
    },
    #[error("{0}")]
    Command(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn printer(name: &str, driver: Option<&str>, port: Option<&str>) -> Printer {
        Printer {
            name: name.to_owned(),
            driver: driver.map(str::to_owned),
            connection: port.map(str::to_owned),
            default: false,
        }
    }

    #[test]
    fn recognizes_printers_that_do_not_print_on_paper() {
        let windows = [
            (
                "Microsoft Print to PDF",
                "Microsoft Print To PDF",
                "PORTPROMPT:",
            ),
            (
                "Microsoft XPS Document Writer",
                "Microsoft XPS Document Writer v4",
                "PORTPROMPT:",
            ),
            ("Fax", "Microsoft Shared Fax Driver", "SHRFAX:"),
            (
                "MS Publisher Color Printer",
                "MS Publisher Color Printer",
                "LPT1:",
            ),
            (
                "OneNote (Desktop)",
                "Send to Microsoft OneNote 16 Driver",
                "nul:",
            ),
            ("Generic / Text Only", "Generic / Text Only", "FILE:"),
        ];
        for (name, driver, port) in windows {
            assert!(
                is_virtual(&printer(name, Some(driver), Some(port))),
                "{name}"
            );
        }
        assert!(is_virtual(&printer("Cups-PDF", None, None)));
    }

    #[test]
    fn keeps_printers_that_print_on_paper() {
        let physical = [
            printer("POS-58", Some("POS-58"), Some("USB002")),
            printer(
                "EPSON TM-T20II",
                Some("EPSON TM-T20II Receipt5"),
                Some("ESDPRT001"),
            ),
            printer(
                "XP-80C",
                Some("Generic / Text Only"),
                Some("IP_192.168.1.50"),
            ),
            printer(
                "Generic / Text Only",
                Some("Generic / Text Only"),
                Some("USB001"),
            ),
            printer("POS_80", None, None),
        ];
        for printer in physical {
            assert!(!is_virtual(&printer), "{}", printer.name);
        }
    }
}
