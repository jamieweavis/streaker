; Inno Setup script that packages Streaker.exe into a Windows installer.
;
; Usage: iscc /DAppVersion=<version> /DSourceExe=<path\to\Streaker.exe> /O<output-dir> scripts\windows-installer.iss
;
; Installs per user (no admin prompt) into %LOCALAPPDATA%\Programs\Streaker. The installer is not
; code signed, so SmartScreen will warn on first run.

#ifndef AppVersion
  #error AppVersion must be defined, e.g. /DAppVersion=3.0.2
#endif
#ifndef SourceExe
  #define SourceExe "..\target\release\Streaker.exe"
#endif

[Setup]
; Never change AppId: upgrades and uninstalls find the existing install by it.
AppId={{357F95EE-A015-4D8A-94E4-4028064FAAC4}
AppName=Streaker
AppVersion={#AppVersion}
AppVerName=Streaker {#AppVersion}
AppPublisher=Jamie Weavis
AppPublisherURL=https://github.com/jamieweavis/streaker
AppSupportURL=https://github.com/jamieweavis/streaker/issues
AppUpdatesURL=https://github.com/jamieweavis/streaker/releases
DefaultDirName={autopf}\Streaker
DefaultGroupName=Streaker
DisableProgramGroupPage=yes
DisableDirPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
LicenseFile=..\LICENSE
SetupIconFile=..\assets\app\icon.ico
UninstallDisplayIcon={app}\Streaker.exe
UninstallDisplayName=Streaker
OutputBaseFilename=Streaker-{#AppVersion}-windows-x64-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
; Streaker lives in the tray with no visible window, so close it without asking when upgrading.
CloseApplications=force
RestartApplications=no

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Streaker"; Filename: "{app}\Streaker.exe"

[Run]
Filename: "{app}\Streaker.exe"; Description: "Launch Streaker"; Flags: nowait postinstall skipifsilent

[Code]
// Remove the "Launch at login" entries the app writes via the auto-launch crate, so an uninstall
// doesn't leave a dangling startup item behind.
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    RegDeleteValue(HKCU, 'Software\Microsoft\Windows\CurrentVersion\Run', 'Streaker');
    RegDeleteValue(HKCU, 'Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run', 'Streaker');
  end;
end;
