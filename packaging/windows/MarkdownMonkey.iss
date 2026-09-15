; MarkdownMonkey Windows installer (Inno Setup)
; 中文：生成可安装到 Program Files 的 Setup.exe
; English: Builds a Setup.exe that installs into Program Files

#ifndef AppVersion
#define AppVersion "0.6.3"
#endif
#ifndef SourceDir
#define SourceDir "..\..\dist\installer-src"
#endif
#ifndef OutputDir
#define OutputDir "..\..\dist"
#endif
#ifndef IconFile
#define IconFile "icon.ico"
#endif

[Setup]
AppId={{8F3C1B2A-6D4E-4A91-9C55-2B7E1A0D4F18}
AppName=MarkdownMonkey
AppVersion={#AppVersion}
AppPublisher=MarkdownMonkey
AppPublisherURL=https://github.com/Elemonbee/dioxus-markdownmonkey
DefaultDirName={autopf}\MarkdownMonkey
DefaultGroupName=MarkdownMonkey
DisableProgramGroupPage=yes
LicenseFile={#SourceDir}\LICENSE
SetupIconFile={#IconFile}
UninstallDisplayIcon={app}\MarkdownMonkey.exe
OutputDir={#OutputDir}
OutputBaseFilename=MarkdownMonkey-{#AppVersion}-windows-x64-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayName=MarkdownMonkey

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional icons:"; Flags: unchecked

[Files]
Source: "{#SourceDir}\MarkdownMonkey.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\README_EN.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\MarkdownMonkey"; Filename: "{app}\MarkdownMonkey.exe"
Name: "{autodesktop}\MarkdownMonkey"; Filename: "{app}\MarkdownMonkey.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\MarkdownMonkey.exe"; Description: "Launch MarkdownMonkey"; Flags: nowait postinstall skipifsilent
