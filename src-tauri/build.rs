fn main() {
    // Only the app commands listed here can be invoked from a webview, and only by windows whose
    // capability grants them (see `capabilities/`).
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_settings",
            "set_language",
            "set_ports",
            "restart",
            "get_server_status",
            "install_certificate",
            "remove_certificate",
            "list_sites",
            "list_history",
            "revoke_site",
            "get_permission_request",
            "answer_permission_request",
            "list_printers",
            "set_printer_shared",
            "print_test_page",
            "get_onboarding_reason",
            "show_licenses",
            "open_log_folder",
        ]),
    ))
    .expect("failed to run tauri-build");
    // The executable embeds icons/icon.ico: rebuild it when `npm run icons` regenerates them.
    println!("cargo:rerun-if-changed=icons");
}
