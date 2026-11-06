; Hooks into Tauri's NSIS installer (see `bundle.windows.nsis.installerHooks`).
;
; This file is included before the installer defines PRODUCTNAME, MAINBINARYNAME and its own
; variables, so the functions below use `$(^Name)` and variables of their own; the macros expand
; later, inside the installer's sections, where everything is defined.

!include LogicLib.nsh

; A file the installer cannot write (an antivirus holding it, for example) fails the installation
; instead of being skipped. Updates run silently, where NSIS would otherwise skip it and report
; success: the old program would stay while the registry and the uninstaller said the new version.
AllowSkipFiles off

; The installed NanoPrint, known once the installation starts.
Var NanoPrintExe
; Whether the desktop shortcut existed when the installer started.
Var NanoPrintHadDesktopShortcut
; The start-with-the-system entry when the installer started, or empty.
Var NanoPrintAutostart
; Set by NanoPrintExeInUse: 1 while some process holds NanoPrint's executable.
Var NanoPrintExeBusy
; Seconds an update has waited for the executable to be free.
Var NanoPrintWaited

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

; Asks Restart Manager, as the installer's own check does, whether a process holds the executable.
Function NanoPrintExeInUse
  StrCpy $NanoPrintExeBusy 0
  !insertmacro RestartManager_StartSession $R0
  ${If} $R0 != ""
    !insertmacro RestartManager_RegisterFile $R0 "$NanoPrintExe"
    ${If} $0 = 0
      System::Call 'RSTRTMGR::RmGetList(p R0, *i .r1, *i .r2, p 0, *i .r3) i .r0'
      ${IfThen} $0 = ${ERROR_MORE_DATA} ${|} StrCpy $NanoPrintExeBusy 1 ${|}
    ${EndIf}
    !insertmacro RestartManager_EndSession $R0
  ${EndIf}
FunctionEnd

; The installer then checks whether anything holds the executable and fails if it cannot close it.
; During an update NanoPrint has just exited, and an antivirus may still be scanning the file; an
; antivirus runs as a service the installer cannot close. Both let go within seconds, so an update
; waits up to 30 seconds for the executable to be free before that check.
!macro NSIS_HOOK_PREINSTALL
  StrCpy $NanoPrintExe "$INSTDIR\${MAINBINARYNAME}.exe"
  ${If} $UpdateMode = 1
    StrCpy $NanoPrintWaited 0
    ${Do}
      Call NanoPrintExeInUse
      ${If} $NanoPrintExeBusy = 0
      ${OrIf} $NanoPrintWaited >= 30
        ${ExitDo}
      ${EndIf}
      Sleep 1000
      IntOp $NanoPrintWaited $NanoPrintWaited + 1
    ${Loop}
  ${EndIf}
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
