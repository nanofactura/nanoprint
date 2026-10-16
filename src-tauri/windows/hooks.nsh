; Hooks into Tauri's NSIS installer (see `bundle.windows.nsis.installerHooks`).

; Runs before the uninstaller removes NanoPrint's files. When the user also deletes NanoPrint's
; data, and this is not an update, NanoPrint removes what deleting files cannot: its certificate
; authorities from the trust store and its signing key from the credential store. Keeping the data
; keeps the certificate trusted, for a reinstall. Start with the system is removed by the
; uninstaller itself.
!macro NSIS_HOOK_PREUNINSTALL
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --uninstall'
  ${EndIf}
!macroend
