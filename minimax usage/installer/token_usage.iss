; Inno Setup 脚本 —— token usage 安装包
; 编译命令: ISCC.exe token_usage.iss

#define MyAppName "token usage"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "token usage"
#define MyAppURL "https://github.com/"
#define MyAppExeName "token_usage.exe"

[Setup]
; 唯一标识符(token usage 专用,跟旧 minimax usage 不冲突)
AppId={{D7E4A1B9-3C2F-4A8E-B6D1-9F2C5E8A4B7C}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
; 默认安装目录:Program Files 下的应用目录
DefaultDirName={autopf}\token usage
; 安装时允许用户改路径(我们要求的!)
DisableProgramGroupPage=yes
; 输出文件
OutputDir=dist
OutputBaseFilename=token_usage_setup_{#MyAppVersion}
; 图标(后续可加)
; SetupIconFile=installer.ico
; 压缩
Compression=lzma2/ultra64
SolidCompression=yes
; 卸载图标
UninstallDisplayIcon={app}\{#MyAppExeName}
; 权限:管理员安装到 Program Files 会显示目录选择页
PrivilegesRequired=admin
PrivilegesRequiredOverridesAllowed=dialog
; 美化
WizardStyle=modern
; 许可协议(跳过)
LicenseFile=
InfoBeforeFile=
InfoAfterFile=

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; 主程序
Source: "token_usage.exe"; DestDir: "{app}"; Flags: ignoreversion
; README
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion isreadme; Check: FileExists('..\README.md')

[Icons]
; 开始菜单
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\设置"; Filename: "{app}\{#MyAppExeName}"; Parameters: "--settings"
Name: "{group}\立即刷新用量"; Filename: "{app}\{#MyAppExeName}"; Parameters: "--console"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
; 桌面图标(可选)
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
; 安装完问用户是否立即启动
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#MyAppName}}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; 卸载时清理临时文件,保留用户配置
Type: filesandordirs; Name: "{userappdata}\token usage\*.tmp"

[Code]
procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    // 配置目录在 %APPDATA%\token usage\ 程序首次运行时会自动创建
    // 这里不用做什么
  end;
end;
