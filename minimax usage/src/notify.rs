//! Toast / balloon 通知 —— 用 PowerShell 弹窗,跨平台稳定
//!
//! 原因:自己用 Win32 NotifyIcon 弹通知时,跟 tray-icon 已注册的 hidden window class
//! 冲突,CreateWindowExW 一直返回 ERROR_MOD_NOT_FOUND。改用 PowerShell 自带的
//! BurntToast / 直接用 Windows Forms NotifyIcon,稳定 0 依赖。

/// 弹一条 toast / balloon 通知。
/// 在 PowerShell 不可用时 fallback 到 stderr。
pub fn show(title: &str, message: &str) {
    // PowerShell 脚本:用 [System.Windows.Forms.NotifyIcon] 弹 balloon
    // Win10/11 会自动把 balloon 转成右下角 toast。
    let script = format!(
        r#"Add-Type -AssemblyName System.Windows.Forms
$ni = New-Object System.Windows.Forms.NotifyIcon
$ni.Icon = [System.Drawing.SystemIcons]::Information
$ni.BalloonTipIcon = [System.Windows.Forms.ToolTipIcon]::Info
$ni.BalloonTipTitle = '{title}'
$ni.BalloonTipText = '{message}'
$ni.Visible = $true
$ni.ShowBalloonTip(8000)
Start-Sleep -Milliseconds 8500
$ni.Visible = $false
$ni.Dispose()
"#,
        title = title.replace('\'', "''"),
        message = message.replace('\'', "''").replace('\n', " ").replace('\r', " ")
    );

    // 写 UTF-8 BOM(中文 Windows 上 PowerShell 5.1 默认按 GBK 读无 BOM 文件)
    let mut bytes = Vec::with_capacity(script.len() + 3);
    bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    bytes.extend_from_slice(script.as_bytes());

    let temp = std::env::temp_dir().join("token_usage_notify.ps1");
    if std::fs::write(&temp, &bytes).is_err() {
        crate::main_log(&format!("通知写文件失败"));
        return;
    }

    let result = std::process::Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-WindowStyle")
        .arg("Hidden")
        .arg("-File")
        .arg(&temp)
        .spawn();

    if let Err(e) = result {
        crate::main_log(&format!("通知 PowerShell 启动失败: {}", e));
    }
}
