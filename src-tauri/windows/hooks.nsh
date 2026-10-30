; Hooks into Tauri's NSIS installer (see `bundle.windows.nsis.installerHooks`).
;
; This file is included before the installer defines PRODUCTNAME, MAINBINARYNAME and its own
; variables, so the functions below use `$(^Name)` and variables of their own; the macros expand
; later, inside the installer's sections, where everything is defined.

!include LogicLib.nsh

; The installed NanoPrint, known once the installation starts.
Var NanoPrintExe
; Whether the desktop shortcut existed when the installer started.
Var NanoPrintHadDesktopShortcut
; The start-with-the-system entry when the installer started, or empty.
Var NanoPrintAutostart

; The finish page offers the desktop shortcut unchecked: NanoPrint lives in the tray and starts with
; the system. An upgrade keeps the shortcut if the user had it (see NSIS_HOOK_POSTINSTALL).
!define MUI_FINISHPAGE_SHOWREADME_NOTCHECKED
!define MUI_CUSTOMFUNCTION_GUIINIT NanoPrintGuiInit

; Runs before an upgrade uninstalls the previous version, which removes its shortcuts and its
; start-with-the-system entry.
Function NanoPrintGuiInit
  StrCpy $NanoPrintHadDesktopShortcut 0
  ${If} ${FileExists} "$DESKTOP\$(^Name).lnk"
    StrCpy $NanoPrintHadDesktopShortcut 1
  ${EndIf}
  ReadRegStr $NanoPrintAutostart HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "$(^Name)"
FunctionEnd

; An update that fails (an antivirus blocking it, for example) leaves NanoPrint closed, because the
; updater exits before the installer runs and only a successful installation starts it again.
; Start the installed version, with the same arguments a successful update would use, so it keeps
; printing and tries the update again later. NanoPrint installs per user and the updater runs the
; installer unelevated, so a plain Exec starts it as the user (Tauri's RunAsUser plugin is not
; loaded yet where this file is included).
Function .onInstFailed
  ClearErrors
  ${GetOptions} $CMDLINE "/UPDATE" $0
  ${IfNot} ${Errors}
    ClearErrors
    ${GetOptions} $CMDLINE "/R" $0
    ${IfNot} ${Errors}
    ${AndIf} $NanoPrintExe != ""
    ${AndIf} ${FileExists} "$NanoPrintExe"
      ${GetOptions} $CMDLINE "/ARGS" $0
      Exec '"$NanoPrintExe" $0'
    ${EndIf}
  ${EndIf}
FunctionEnd

!macro NSIS_HOOK_PREINSTALL
  StrCpy $NanoPrintExe "$INSTDIR\${MAINBINARYNAME}.exe"
!macroend

; Uninstalling the previous version during an upgrade removed its desktop shortcut and its
; start-with-the-system entry: put back the ones the user had. Updates never uninstall, so they
; keep both as they are.
!macro NSIS_HOOK_POSTINSTALL
  ${If} $NanoPrintHadDesktopShortcut = 1
  ${AndIfNot} ${FileExists} "$DESKTOP\${PRODUCTNAME}.lnk"
    Call CreateOrUpdateDesktopShortcut
  ${EndIf}
  ${If} $NanoPrintAutostart != ""
    ClearErrors
    ReadRegStr $0 HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCTNAME}"
    ${If} ${Errors}
      WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCTNAME}" "$NanoPrintAutostart"
    ${EndIf}
  ${EndIf}
!macroend

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
