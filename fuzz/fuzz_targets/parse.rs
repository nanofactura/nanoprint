//! Arbitrary text frames into the parser of everything a web page can send.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = nanoprint_protocol::parse(text);
    }
});
