//! The Windows print spooler (winspool). This module holds the only `unsafe` FFI calls; each one
//! documents why it is sound.
#![allow(unsafe_code)]

use std::{io, ptr, slice};

use windows_sys::Win32::{
    Foundation::ERROR_INSUFFICIENT_BUFFER,
    Graphics::Printing::{
        AbortPrinter, ClosePrinter, DOC_INFO_1W, EndDocPrinter, EndPagePrinter, EnumPrintersW,
        GetDefaultPrinterW, OpenPrinterW, PRINTER_ENUM_CONNECTIONS, PRINTER_ENUM_LOCAL,
        PRINTER_HANDLE, PRINTER_INFO_2W, StartDocPrinterW, StartPagePrinter, WritePrinter,
    },
};

use super::Error;
use crate::server::Printer;

/// Local printers and connections to shared printers.
pub fn list() -> Result<Vec<Printer>, Error> {
    let flags = PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS;
    let default = default_printer();
    let mut needed: u32 = 0;
    let mut count: u32 = 0;
    // Retried because a printer can be added between the size query and the read.
    for _ in 0..3 {
        // PRINTER_INFO_2W structs followed by their strings; `u64` keeps the buffer aligned.
        let mut buffer = vec![0_u64; (needed as usize).div_ceil(8)];
        let size = u32::try_from(buffer.len() * 8).unwrap_or(u32::MAX);
        let target = if buffer.is_empty() {
            ptr::null_mut()
        } else {
            buffer.as_mut_ptr().cast::<u8>()
        };
        // SAFETY: `target` is null with a size of 0, or writable for `size` bytes; the out
        // pointers are valid for the duration of the call.
        let ok = unsafe {
            EnumPrintersW(
                flags,
                ptr::null(),
                2,
                target,
                size,
                &raw mut needed,
                &raw mut count,
            )
        };
        if ok != 0 {
            // SAFETY: on success the buffer starts with `count` initialized, aligned structs.
            let infos = unsafe {
                slice::from_raw_parts(buffer.as_ptr().cast::<PRINTER_INFO_2W>(), count as usize)
            };
            return Ok(infos
                .iter()
                .filter_map(|info| {
                    // SAFETY: the strings point into `buffer`, alive for this whole closure.
                    let (name, driver, port) = unsafe {
                        (
                            wide_to_string(info.pPrinterName),
                            wide_to_string(info.pDriverName),
                            wide_to_string(info.pPortName),
                        )
                    };
                    let name = name?;
                    Some(Printer {
                        default: default.as_deref() == Some(name.as_str()),
                        name,
                        driver,
                        connection: port,
                    })
                })
                .collect());
        }
        let error = io::Error::last_os_error();
        if error
            .raw_os_error()
            .and_then(|code| u32::try_from(code).ok())
            != Some(ERROR_INSUFFICIENT_BUFFER)
        {
            return Err(Error::Os {
                operation: "EnumPrinters",
                source: error,
            });
        }
    }
    Err(Error::Command("the printer list kept changing".into()))
}

/// Sends `data` as one RAW document, bypassing the driver's rendering.
pub fn print_raw(printer: &str, job_name: &str, data: &[u8]) -> Result<(), Error> {
    let name = to_wide(printer);
    let mut handle = PRINTER_HANDLE::default();
    // SAFETY: `name` is NUL-terminated and `handle` is a valid out pointer; no defaults passed.
    if unsafe { OpenPrinterW(name.as_ptr(), &raw mut handle, ptr::null()) } == 0 {
        return Err(os_error("OpenPrinter"));
    }
    let printer = OpenPrinter(handle);

    let mut doc_name = to_wide(job_name);
    let mut datatype = to_wide("RAW");
    let info = DOC_INFO_1W {
        pDocName: doc_name.as_mut_ptr(),
        pOutputFile: ptr::null_mut(),
        pDatatype: datatype.as_mut_ptr(),
    };
    // SAFETY: `printer` is open; `info` and the strings it points to outlive the call.
    if unsafe { StartDocPrinterW(printer.0, 1, &raw const info) } == 0 {
        return Err(os_error("StartDocPrinter"));
    }
    if let Err(error) = write_page(printer.0, data) {
        // Cancel the document: ending it would queue the part already written, and a cut-off
        // ESC/POS stream prints a broken receipt (the page retries anyway).
        // SAFETY: a document was started on this open handle.
        unsafe { AbortPrinter(printer.0) };
        return Err(error);
    }
    // SAFETY: a document was started on this open handle.
    if unsafe { EndDocPrinter(printer.0) } == 0 {
        return Err(os_error("EndDocPrinter"));
    }
    Ok(())
}

fn write_page(printer: PRINTER_HANDLE, data: &[u8]) -> Result<(), Error> {
    // SAFETY: `printer` is open with a started document.
    if unsafe { StartPagePrinter(printer) } == 0 {
        return Err(os_error("StartPagePrinter"));
    }
    let mut offset = 0;
    while offset < data.len() {
        let chunk = &data[offset..];
        let size = u32::try_from(chunk.len()).unwrap_or(u32::MAX);
        let mut written: u32 = 0;
        // SAFETY: `chunk` is readable for `size` bytes and `written` is a valid out pointer.
        let ok = unsafe { WritePrinter(printer, chunk.as_ptr().cast(), size, &raw mut written) };
        if ok == 0 || written == 0 {
            return Err(os_error("WritePrinter"));
        }
        offset += written as usize;
    }
    // SAFETY: a page was started on this handle.
    if unsafe { EndPagePrinter(printer) } == 0 {
        return Err(os_error("EndPagePrinter"));
    }
    Ok(())
}

fn default_printer() -> Option<String> {
    let mut len: u32 = 0;
    // SAFETY: a null buffer only queries the required length.
    unsafe { GetDefaultPrinterW(ptr::null_mut(), &raw mut len) };
    if len == 0 {
        return None;
    }
    let mut buffer = vec![0_u16; len as usize];
    // SAFETY: `buffer` holds `len` UTF-16 units, as `len` tells the function.
    if unsafe { GetDefaultPrinterW(buffer.as_mut_ptr(), &raw mut len) } == 0 {
        return None;
    }
    // SAFETY: on success the buffer holds a NUL-terminated string.
    unsafe { wide_to_string(buffer.as_mut_ptr()) }
}

/// Closes the printer handle when dropped.
struct OpenPrinter(PRINTER_HANDLE);

impl Drop for OpenPrinter {
    fn drop(&mut self) {
        // SAFETY: the handle came from a successful OpenPrinterW and is closed exactly once.
        unsafe { ClosePrinter(self.0) };
    }
}

fn to_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

/// # Safety
///
/// `text` must be null or point to a NUL-terminated UTF-16 string.
unsafe fn wide_to_string(text: *mut u16) -> Option<String> {
    if text.is_null() {
        return None;
    }
    let mut len = 0;
    // SAFETY: the caller guarantees a terminating NUL, so every unit up to it is readable.
    while unsafe { *text.add(len) } != 0 {
        len += 1;
    }
    // SAFETY: the `len` units before the NUL were just read.
    let units = unsafe { slice::from_raw_parts(text, len) };
    Some(String::from_utf16_lossy(units))
}

fn os_error(operation: &'static str) -> Error {
    Error::Os {
        operation,
        source: io::Error::last_os_error(),
    }
}
