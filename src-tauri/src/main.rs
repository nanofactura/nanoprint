// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
mod rendering;

fn main() {
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    if let Err(error) = rendering::configure() {
        report_rendering_error(&error);
    }

    nanoprint_lib::run();
}

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
#[allow(clippy::print_stderr)] // The fallback runs before the application logger is initialized.
fn report_rendering_error(error: &std::io::Error) {
    eprintln!("NanoPrint: could not enable the Raspberry Pi rendering fallback: {error}");
}
