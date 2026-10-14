Unicode true
RequestExecutionLevel user
ManifestDPIAware true
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "WinVer.nsh"
!include "x64.nsh"
Name "Beni Buddy"
OutFile "${OUTPUT}"
InstallDir "$LOCALAPPDATA\Programs\Beni Buddy"
InstallDirRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\io.witzelfitz.beni-buddy" "InstallLocation"
SetCompressor /SOLID lzma
SetOverwrite on
VIProductVersion "${VERSION}.0"
VIAddVersionKey /LANG=1031 "ProductName" "Beni Buddy"
VIAddVersionKey /LANG=1031 "CompanyName" "Witzelfitz Consulting GmbH"
VIAddVersionKey /LANG=1031 "FileDescription" "Beni Buddy Installer"
VIAddVersionKey /LANG=1031 "FileVersion" "${VERSION}"
VIAddVersionKey /LANG=1031 "LegalCopyright" "Rechte gemäss mitgelieferten Lizenztexten"
!define MUI_ICON "${ICON}"
!define MUI_UNICON "${ICON}"
!define MUI_ABORTWARNING
!define MUI_FINISHPAGE_RUN "$INSTDIR\beni-desktop-buddy.exe"
!define MUI_FINISHPAGE_RUN_NOTCHECKED
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "${LICENSE}"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "German"

Function .onInit
  ${IfNot} ${RunningX64}
    MessageBox MB_OK|MB_ICONSTOP "Beni Buddy benötigt Windows auf einem 64-Bit-Intel-/AMD-Prozessor."
    SetErrorLevel 2
    Quit
  ${EndIf}
  ${IfNot} ${AtLeastWin11}
    MessageBox MB_OK|MB_ICONSTOP "Dieser Installer benötigt Windows 11 oder neuer."
    SetErrorLevel 2
    Quit
  ${EndIf}
  SetShellVarContext current
FunctionEnd

!macro CheckExecutableAvailable
  IfFileExists "$INSTDIR\beni-desktop-buddy.exe" 0 available
  System::Call 'kernel32::CreateFileW(w "$INSTDIR\beni-desktop-buddy.exe", i 0x40000000, i 0, p 0, i 3, i 0, p 0) p.r0'
  ${If} $0 == -1
    IfSilent +2
    MessageBox MB_OK|MB_ICONEXCLAMATION "Bitte Beni über sein Menü beenden und den Schreibzugriff auf den Installationsordner prüfen."
    SetErrorLevel 2
    Quit
  ${EndIf}
  System::Call 'kernel32::CloseHandle(p r0)'
  available:
!macroend

Section "Beni Buddy"
  !insertmacro CheckExecutableAvailable
  !include "${INSTALL_FILES}"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  CreateDirectory "$SMPROGRAMS\Beni Buddy"
  CreateShortcut "$SMPROGRAMS\Beni Buddy\Beni Buddy.lnk" "$INSTDIR\beni-desktop-buddy.exe"
  CreateShortcut "$SMPROGRAMS\Beni Buddy\Deinstallieren.lnk" "$INSTDIR\Uninstall.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\io.witzelfitz.beni-buddy" "DisplayName" "Beni Buddy"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\io.witzelfitz.beni-buddy" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\io.witzelfitz.beni-buddy" "Publisher" "Witzelfitz Consulting GmbH"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\io.witzelfitz.beni-buddy" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\io.witzelfitz.beni-buddy" "DisplayIcon" "$INSTDIR\beni-desktop-buddy.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\io.witzelfitz.beni-buddy" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\io.witzelfitz.beni-buddy" "QuietUninstallString" '"$INSTDIR\Uninstall.exe" /S'
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\io.witzelfitz.beni-buddy" "NoModify" 1
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\io.witzelfitz.beni-buddy" "NoRepair" 1
SectionEnd

Section "Uninstall"
  SetShellVarContext current
  !insertmacro CheckExecutableAvailable
  !include "${REMOVE_FILES}"
  Delete "$INSTDIR\Uninstall.exe"
  Delete "$SMPROGRAMS\Beni Buddy\Beni Buddy.lnk"
  Delete "$SMPROGRAMS\Beni Buddy\Deinstallieren.lnk"
  RMDir "$SMPROGRAMS\Beni Buddy"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\io.witzelfitz.beni-buddy"
  RMDir "$INSTDIR"
SectionEnd
