//! A process-local fallback for Raspberry Pi's `WebKitGTK` DMA-BUF rendering corruption.
//! Keep it scoped to Pi Wayland: disabling DMA-BUF can break rendering on other graphics stacks.

use std::{ffi::OsStr, io::Write, os::unix::process::CommandExt, process::Command};

const DMABUF: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";
const MODEL: &str = "/sys/firmware/devicetree/base/model";

/// Runs before Tauri or any worker thread. Re-exec avoids Rust 2024's unsafe process-wide
/// environment mutation and gives `WebKit`'s child processes the same renderer setting.
pub fn configure() -> std::io::Result<()> {
    let model = model_or_warn(std::fs::read(MODEL), &mut std::io::stderr().lock());
    if needs_fallback(
        &model,
        std::env::var_os("WAYLAND_DISPLAY").as_deref(),
        std::env::var_os("GDK_BACKEND").as_deref(),
        std::env::var_os(DMABUF).as_deref(),
    ) {
        let error = Command::new(std::env::current_exe()?)
            .args(std::env::args_os().skip(1))
            .env(DMABUF, "1")
            .exec();
        return Err(error);
    }
    Ok(())
}

fn model_or_warn(result: std::io::Result<Vec<u8>>, diagnostic: &mut impl Write) -> Vec<u8> {
    match result {
        Ok(model) => model,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => {
            let _ = writeln!(
                diagnostic,
                "NanoPrint: could not read Raspberry Pi model from {MODEL}: {error}; \
                 skipping automatic DMA-BUF fallback"
            );
            Vec::new()
        }
    }
}

fn needs_fallback(
    model: &[u8],
    wayland_display: Option<&OsStr>,
    gdk_backend: Option<&OsStr>,
    renderer_override: Option<&OsStr>,
) -> bool {
    model.starts_with(b"Raspberry Pi ")
        && wayland_display.is_some_and(|display| !display.is_empty())
        && gdk_backend.is_none_or(|backend| {
            backend.is_empty()
                || backend
                    .to_str()
                    .is_some_and(|value| value.split(',').next() == Some("wayland"))
        })
        && renderer_override.is_none()
}

#[cfg(test)]
mod tests {
    use super::{MODEL, model_or_warn, needs_fallback};
    use std::ffi::OsStr;

    const PI: &[u8] = b"Raspberry Pi 400 Rev 1.0\0";

    #[test]
    fn warns_about_model_read_errors_except_missing_files_and_continues() {
        use std::io::{Error, ErrorKind};

        let mut diagnostic = Vec::new();
        assert_eq!(
            model_or_warn(Err(ErrorKind::NotFound.into()), &mut diagnostic),
            Vec::<u8>::new()
        );
        assert_eq!(diagnostic, Vec::<u8>::new());

        for kind in [ErrorKind::PermissionDenied, ErrorKind::InvalidData] {
            diagnostic.clear();
            let model = model_or_warn(Err(Error::new(kind, "model read failed")), &mut diagnostic);
            assert!(!needs_fallback(
                &model,
                Some(OsStr::new("wayland-0")),
                None,
                None
            ));
            let message = std::str::from_utf8(&diagnostic).unwrap();
            assert!(message.contains(MODEL));
            assert!(message.contains("model read failed"));
            assert!(message.contains("skipping automatic DMA-BUF fallback"));
        }
    }

    #[test]
    fn limits_fallback_to_raspberry_pi_using_wayland() {
        let display = Some(OsStr::new("wayland-0"));
        assert!(needs_fallback(PI, display, None, None));
        assert!(needs_fallback(
            PI,
            display,
            Some(OsStr::new("wayland,x11")),
            None
        ));
        assert!(!needs_fallback(b"", display, None, None));
        assert!(!needs_fallback(b"Other ARM board", display, None, None));
        assert!(!needs_fallback(PI, None, None, None));
        assert!(!needs_fallback(PI, Some(OsStr::new("")), None, None));
        assert!(!needs_fallback(PI, display, Some(OsStr::new("x11")), None));
        assert!(!needs_fallback(
            PI,
            display,
            Some(OsStr::new("x11,wayland")),
            None
        ));
    }

    #[test]
    fn preserves_every_explicit_renderer_override() {
        for value in ["0", "1", "", "custom"] {
            assert!(!needs_fallback(
                PI,
                Some(OsStr::new("wayland-0")),
                None,
                Some(OsStr::new(value))
            ));
        }
    }
}
