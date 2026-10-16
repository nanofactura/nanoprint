//! The subset of the qz-tray.js wire protocol that raw printing needs: JSON text frames over a
//! WebSocket. Written from the public client API and black-box traces; see `docs/protocol.md`.
//!
//! This is everything a web page can send to NanoPrint, so it lives apart from the app: it has no
//! I/O, and `fuzz/` exercises it with arbitrary input.

use base64::Engine;
use serde_json::{Value, json};

/// Protocol level reported by `getVersion`. qz-tray.js rewrites its requests into legacy shapes
/// for older servers, so NanoPrint reports the level whose request format it implements.
pub const PROTOCOL_VERSION: &str = "2.3.0";

/// Longest `uid` echoed back to the client. qz-tray.js sends six characters.
const MAX_UID_LEN: usize = 64;

/// Copies per job are capped so a single request cannot waste a roll of paper.
pub const MAX_COPIES: u32 = 10;

/// A frame received from the client.
#[derive(Debug, PartialEq, Eq)]
pub enum Incoming {
    /// Keep-alive (`"ping"`) and probes that get no answer.
    Ignored,
    /// The connection setup message. QZ Tray uses it to receive the site's signing certificate.
    /// NanoPrint asks the user about the page's origin instead, and only reads the field for the
    /// site's optional name and logo (see `identity.rs`).
    Setup {
        uid: String,
        certificate: Option<String>,
    },
    /// An API call.
    Call { uid: String, call: Call },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Call {
    GetVersion {
        details: bool,
    },
    FindPrinters {
        query: Option<String>,
    },
    DefaultPrinter,
    PrinterDetails,
    Print(PrintRequest),
    /// A call NanoPrint does not implement; answered with an error.
    Unsupported(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct PrintRequest {
    pub printer: PrinterTarget,
    pub job_name: Option<String>,
    pub copies: u32,
    /// The bytes to send, already decoded from the request's data elements.
    pub data: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PrinterTarget {
    Name(String),
    /// `printer: {host, port}`: a network printer addressed directly by the page.
    Host {
        host: String,
        port: u16,
    },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The frame is not a JSON object or carries no usable `uid`; the client cannot be answered.
    #[error("malformed message")]
    Malformed,
    /// A call with invalid parameters; answered to the client with this message.
    #[error("{message}")]
    InvalidCall { uid: String, message: String },
}

/// Parses one text frame.
pub fn parse(text: &str) -> Result<Incoming, Error> {
    if text == "ping" || text == "getProgramName" {
        return Ok(Incoming::Ignored);
    }
    let Ok(Value::Object(message)) = serde_json::from_str::<Value>(text) else {
        return Err(Error::Malformed);
    };
    let uid = match message.get("uid") {
        Some(Value::String(uid)) if !uid.is_empty() && uid.len() <= MAX_UID_LEN => uid.clone(),
        _ => return Err(Error::Malformed),
    };
    let Some(call) = message.get("call").and_then(Value::as_str) else {
        return if let Some(certificate) = message.get("certificate") {
            Ok(Incoming::Setup {
                uid,
                certificate: certificate.as_str().map(str::to_owned),
            })
        } else {
            Err(Error::Malformed)
        };
    };
    let params = message.get("params").unwrap_or(&Value::Null);
    let call = match call {
        "getVersion" => Call::GetVersion {
            details: params.get("details").and_then(Value::as_bool) == Some(true),
        },
        "printers.find" => Call::FindPrinters {
            query: params
                .get("query")
                .and_then(Value::as_str)
                .filter(|query| !query.is_empty())
                .map(str::to_owned),
        },
        "printers.getDefault" => Call::DefaultPrinter,
        "printers.detail" => Call::PrinterDetails,
        "print" => Call::Print(print_request(params).map_err(|message| Error::InvalidCall {
            uid: uid.clone(),
            message,
        })?),
        other => Call::Unsupported(other.chars().take(64).collect()),
    };
    Ok(Incoming::Call { uid, call })
}

/// `{uid, result}`.
pub fn result(uid: &str, result: &Value) -> String {
    json!({ "uid": uid, "result": result }).to_string()
}

/// `{uid, error}`.
pub fn error(uid: &str, message: &str) -> String {
    json!({ "uid": uid, "error": message }).to_string()
}

fn print_request(params: &Value) -> Result<PrintRequest, String> {
    let printer = params.get("printer").ok_or("Missing printer")?;
    let printer = match (printer.get("name"), printer.get("host")) {
        (Some(Value::String(name)), _) if !name.is_empty() => PrinterTarget::Name(name.clone()),
        (_, Some(Value::String(host))) if !host.is_empty() => PrinterTarget::Host {
            host: host.clone(),
            port: printer
                .get("port")
                .and_then(Value::as_u64)
                .and_then(|port| u16::try_from(port).ok())
                .unwrap_or(9100),
        },
        _ => return Err("Missing printer name".into()),
    };
    let options = params.get("options").unwrap_or(&Value::Null);
    let encoding = match options.get("encoding") {
        None | Some(Value::Null) => None,
        Some(Value::String(name)) => Some(name.as_str()),
        // qz-tray.js also takes `{from, to}`. The text arrives as Unicode, so only `to`, the
        // printer's encoding, matters.
        Some(Value::Object(pair)) => Some(
            pair.get("to")
                .and_then(Value::as_str)
                .ok_or("Unsupported encoding: an encoding object needs 'to'")?,
        ),
        Some(_) => return Err("Unsupported encoding".into()),
    };
    let copies = options
        .get("copies")
        .and_then(Value::as_u64)
        .map_or(1, |copies| copies.clamp(1, u64::from(MAX_COPIES)));
    let job_name = options
        .get("jobName")
        .and_then(Value::as_str)
        .map(sanitize_job_name)
        .filter(|name| !name.is_empty());

    let elements = params
        .get("data")
        .and_then(Value::as_array)
        .ok_or("Missing print data")?;
    let mut data = Vec::new();
    for element in elements {
        data.extend(decode_element(element, encoding)?);
    }
    if data.is_empty() {
        return Err("Empty print data".into());
    }
    Ok(PrintRequest {
        printer,
        job_name,
        copies: u32::try_from(copies).unwrap_or(1),
        data,
    })
}

/// Decodes one element of `data`: a plain string, or `{type:'raw', format:'command', flavor, data}`.
fn decode_element(element: &Value, encoding: Option<&str>) -> Result<Vec<u8>, String> {
    let (kind, format, flavor, data) = match element {
        Value::String(text) => ("raw", "command", "plain", text.as_str()),
        Value::Object(object) => {
            let field = |name| object.get(name).and_then(Value::as_str);
            (
                field("type").unwrap_or("raw"),
                field("format").unwrap_or("command"),
                field("flavor").unwrap_or("plain"),
                field("data").ok_or("Missing data in print element")?,
            )
        }
        _ => return Err("Invalid print element".into()),
    };
    if !kind.eq_ignore_ascii_case("raw") || !format.eq_ignore_ascii_case("command") {
        return Err(format!(
            "Only raw command data is supported (got type '{kind}', format '{format}')"
        ));
    }
    match flavor.to_ascii_lowercase().as_str() {
        "plain" => encode_text(data, encoding),
        "base64" => {
            let compact: String = data.chars().filter(|c| !c.is_whitespace()).collect();
            base64::engine::general_purpose::STANDARD
                .decode(compact)
                .map_err(|_| "Invalid base64 data".to_owned())
        }
        "hex" => {
            let compact: String = data.chars().filter(|c| !c.is_whitespace()).collect();
            hex::decode(compact).map_err(|_| "Invalid hex data".to_owned())
        }
        other => Err(format!("Unsupported data flavor '{other}'")),
    }
}

/// Encodes text with a Java-style charset name, as qz-tray.js clients send them. Characters the
/// charset cannot represent become `?`, like Java's encoder.
fn encode_text(text: &str, encoding: Option<&str>) -> Result<Vec<u8>, String> {
    let Some(name) = encoding.map(str::trim).filter(|name| !name.is_empty()) else {
        return Ok(text.as_bytes().to_vec());
    };
    let normalized = name.to_ascii_lowercase().replace('_', "-");
    let bytes = match normalized.as_str() {
        "utf-8" | "utf8" | "legacy" => text.as_bytes().to_vec(),
        "iso-8859-1" | "iso8859-1" | "latin1" | "8859-1" | "cp819" | "ibm819" => {
            text.chars().map(latin1).collect()
        }
        "windows-1252" | "cp1252" | "cp-1252" | "1252" => text.chars().map(windows_1252).collect(),
        "cp437" | "ibm437" | "ibm-437" | "437" => {
            oem_cp::encode_string_lossy(text, &oem_cp::code_table::ENCODING_TABLE_CP437)
        }
        "cp850" | "ibm850" | "ibm-850" | "850" => {
            oem_cp::encode_string_lossy(text, &oem_cp::code_table::ENCODING_TABLE_CP850)
        }
        "cp858" | "ibm858" | "ibm-858" | "ibm00858" | "858" => {
            oem_cp::encode_string_lossy(text, &oem_cp::code_table::ENCODING_TABLE_CP858)
        }
        _ => return Err(format!("Unsupported encoding '{name}'")),
    };
    Ok(bytes)
}

fn latin1(c: char) -> u8 {
    u8::try_from(u32::from(c)).unwrap_or(b'?')
}

/// Windows-1252 is ISO-8859-1 with printable characters in 0x80–0x9F.
fn windows_1252(c: char) -> u8 {
    const HIGH: [(char, u8); 27] = [
        ('€', 0x80),
        ('‚', 0x82),
        ('ƒ', 0x83),
        ('„', 0x84),
        ('…', 0x85),
        ('†', 0x86),
        ('‡', 0x87),
        ('ˆ', 0x88),
        ('‰', 0x89),
        ('Š', 0x8A),
        ('‹', 0x8B),
        ('Œ', 0x8C),
        ('Ž', 0x8E),
        ('‘', 0x91),
        ('’', 0x92),
        ('“', 0x93),
        ('”', 0x94),
        ('•', 0x95),
        ('–', 0x96),
        ('—', 0x97),
        ('˜', 0x98),
        ('™', 0x99),
        ('š', 0x9A),
        ('›', 0x9B),
        ('œ', 0x9C),
        ('ž', 0x9E),
        ('Ÿ', 0x9F),
    ];
    if let Some(&(_, byte)) = HIGH.iter().find(|(high, _)| *high == c) {
        return byte;
    }
    match u32::from(c) {
        // C1 control range: not printable characters in Windows-1252.
        0x80..=0x9F => b'?',
        _ => latin1(c),
    }
}

/// Keeps job names readable in the OS print queue: no control characters, at most 128 chars.
fn sanitize_job_name(name: &str) -> String {
    name.chars()
        .filter(|c| !c.is_control())
        .take(128)
        .collect::<String>()
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(text: &str) -> Call {
        match parse(text) {
            Ok(Incoming::Call { call, .. }) => call,
            other => panic!("expected a call, got {other:?}"),
        }
    }

    #[test]
    fn ignores_keep_alive_and_probes() {
        assert_eq!(parse("ping"), Ok(Incoming::Ignored));
        assert_eq!(parse("getProgramName"), Ok(Incoming::Ignored));
    }

    #[test]
    fn recognizes_the_setup_message() {
        let setup = r#"{"certificate":null,"timestamp":1,"uid":"abc123","position":{"x":0,"y":0}}"#;
        assert_eq!(
            parse(setup),
            Ok(Incoming::Setup {
                uid: "abc123".into(),
                certificate: None,
            })
        );
        let presented = r#"{"certificate":"{\"nanoprint\":{\"name\":\"Kiosco\"}}","uid":"abc123"}"#;
        assert_eq!(
            parse(presented),
            Ok(Incoming::Setup {
                uid: "abc123".into(),
                certificate: Some(r#"{"nanoprint":{"name":"Kiosco"}}"#.into()),
            })
        );
    }

    #[test]
    fn rejects_frames_without_a_usable_uid() {
        assert_eq!(parse("not json"), Err(Error::Malformed));
        assert_eq!(parse(r#"{"call":"getVersion"}"#), Err(Error::Malformed));
        let long_uid = format!(r#"{{"call":"getVersion","uid":"{}"}}"#, "x".repeat(65));
        assert_eq!(parse(&long_uid), Err(Error::Malformed));
    }

    #[test]
    fn parses_printer_calls() {
        assert_eq!(
            call(r#"{"call":"printers.find","params":{},"uid":"a"}"#),
            Call::FindPrinters { query: None }
        );
        assert_eq!(
            call(r#"{"call":"printers.find","params":{"query":"pos"},"uid":"a"}"#),
            Call::FindPrinters {
                query: Some("pos".into())
            }
        );
        assert_eq!(
            call(r#"{"call":"printers.getDefault","params":null,"uid":"a"}"#),
            Call::DefaultPrinter
        );
        assert_eq!(
            call(r#"{"call":"printers.detail","uid":"a"}"#),
            Call::PrinterDetails
        );
        assert_eq!(
            call(r#"{"call":"getVersion","params":{"details":true},"uid":"a"}"#),
            Call::GetVersion { details: true }
        );
        assert_eq!(
            call(r#"{"call":"file.read","params":{"path":"/etc/passwd"},"uid":"a"}"#),
            Call::Unsupported("file.read".into())
        );
    }

    #[test]
    fn decodes_print_data_in_every_flavor() {
        // The exact request qz-tray.js 2.3.0 sends for `qz.print(config, data)`.
        let text = r#"{"call":"print","params":{"printer":{"name":"POS-58"},"options":{"copies":2,"jobName":"Ticket","encoding":"CP850","spool":null},"data":["\u001b@Año\n",{"type":"raw","format":"command","flavor":"base64","data":"G0A="},{"type":"raw","format":"command","flavor":"hex","data":"1b 40"}]},"uid":"a","signature":"","signAlgorithm":"SHA1"}"#;
        let Call::Print(request) = call(text) else {
            panic!("expected a print call")
        };
        assert_eq!(request.printer, PrinterTarget::Name("POS-58".into()));
        assert_eq!(request.job_name.as_deref(), Some("Ticket"));
        assert_eq!(request.copies, 2);
        // "ñ" is 0xA4 in code page 850.
        assert_eq!(
            request.data,
            [0x1b, b'@', b'A', 0xA4, b'o', b'\n', 0x1b, b'@', 0x1b, b'@']
        );
    }

    #[test]
    fn encodes_text_with_java_charset_names() {
        assert_eq!(encode_text("ñ", None), Ok("ñ".as_bytes().to_vec()));
        assert_eq!(encode_text("ñ", Some("ISO-8859-1")), Ok(vec![0xF1]));
        assert_eq!(encode_text("€ñ", Some("Cp1252")), Ok(vec![0x80, 0xF1]));
        assert_eq!(encode_text("ñ", Some("IBM437")), Ok(vec![0xA4]));
        assert_eq!(encode_text("€", Some("cp858")), Ok(vec![0xD5]));
        assert_eq!(encode_text("✓", Some("ISO-8859-1")), Ok(vec![b'?']));
        assert!(encode_text("a", Some("EBCDIC")).is_err());
    }

    #[test]
    fn takes_the_target_of_an_encoding_object() {
        let print = |encoding: &str| {
            parse(&format!(
                r#"{{"call":"print","params":{{"printer":{{"name":"P"}},"options":{{"encoding":{encoding}}},"data":["ñ"]}},"uid":"a"}}"#
            ))
        };
        let Ok(Incoming::Call {
            call: Call::Print(request),
            ..
        }) = print(r#"{"from":"UTF-8","to":"Cp850"}"#)
        else {
            panic!("expected a print call")
        };
        assert_eq!(request.data, [0xA4]);
        // Without a target, or in a form qz-tray.js never sends, the job is refused rather than
        // printed in the wrong encoding.
        for unsupported in [r#"{"from":"UTF-8"}"#, "850", "true", "[]"] {
            assert!(
                matches!(print(unsupported), Err(Error::InvalidCall { .. })),
                "{unsupported}"
            );
        }
        let Ok(Incoming::Call {
            call: Call::Print(request),
            ..
        }) = print("null")
        else {
            panic!("expected a print call")
        };
        assert_eq!(request.data, "ñ".as_bytes());
    }

    #[test]
    fn rejects_invalid_print_requests() {
        let request = |params: &str| {
            parse(&format!(
                r#"{{"call":"print","params":{params},"uid":"u1"}}"#
            ))
        };
        let invalid = |message: &str| {
            Err(Error::InvalidCall {
                uid: "u1".into(),
                message: message.into(),
            })
        };
        assert_eq!(request(r#"{"data":["x"]}"#), invalid("Missing printer"));
        assert_eq!(
            request(r#"{"printer":{"name":"P"},"data":[]}"#),
            invalid("Empty print data")
        );
        assert_eq!(
            request(
                r#"{"printer":{"name":"P"},"data":[{"type":"pixel","format":"html","data":"<b>"}]}"#
            ),
            invalid("Only raw command data is supported (got type 'pixel', format 'html')")
        );
        assert_eq!(
            request(
                r#"{"printer":{"name":"P"},"data":[{"type":"raw","flavor":"file","data":"C:/x"}]}"#
            ),
            invalid("Unsupported data flavor 'file'")
        );
    }

    #[test]
    fn caps_copies_and_cleans_job_names() {
        let text = r#"{"call":"print","params":{"printer":{"name":"P"},"options":{"copies":500,"jobName":"\u0007 Ticket \n"},"data":["x"]},"uid":"a"}"#;
        let Call::Print(request) = call(text) else {
            panic!("expected a print call")
        };
        assert_eq!(request.copies, MAX_COPIES);
        assert_eq!(request.job_name.as_deref(), Some("Ticket"));
    }

    #[test]
    fn builds_responses() {
        let parsed = |text: String| serde_json::from_str::<Value>(&text).unwrap();
        assert_eq!(
            parsed(result("a", &json!(null))),
            json!({"uid": "a", "result": null})
        );
        assert_eq!(
            parsed(error("a", "nope")),
            json!({"uid": "a", "error": "nope"})
        );
    }
}

/// Whatever a page sends, parsing returns a result and never panics. These run on every build;
/// `fuzz/` explores much further with coverage guidance.
#[cfg(test)]
mod properties {
    use proptest::prelude::*;
    use serde_json::json;

    use super::*;

    proptest! {
        #[test]
        fn never_panics_on_any_text(text in ".*") {
            let _ = parse(&text);
        }

        #[test]
        fn never_panics_on_print_requests(
            printer in ".*",
            encoding in proptest::option::of(".*"),
            flavor in "(plain|base64|hex|file|[a-zA-Z]{0,8})",
            data in ".*",
            copies in any::<i64>(),
            job_name in ".*",
        ) {
            let message = json!({
                "call": "print",
                "uid": "u",
                "params": {
                    "printer": { "name": printer },
                    "options": { "encoding": encoding, "copies": copies, "jobName": job_name },
                    "data": [data.clone(), { "type": "raw", "format": "command", "flavor": flavor, "data": data }],
                },
            });
            if let Ok(Incoming::Call { call: Call::Print(request), .. }) = parse(&message.to_string()) {
                prop_assert!((1..=MAX_COPIES).contains(&request.copies));
                prop_assert!(!request.data.is_empty());
            }
        }

        #[test]
        fn echoes_uids_as_valid_json(uid in ".{1,64}", message in ".*") {
            let reply: serde_json::Value = serde_json::from_str(&error(&uid, &message)).unwrap();
            prop_assert_eq!(&reply["uid"], &json!(uid));
        }
    }
}
