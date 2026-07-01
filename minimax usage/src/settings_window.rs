//! 设置 UI —— 启动 PowerShell + WinForms 对话框
//!
//! Win32 手写控件经常因空字符串 PCWSTR 崩溃,Win32 原生控件在 Win11 也丑。
//! 改用 PowerShell 自带的 Windows Forms(现代深色样式),0 依赖,跨 Rust 进程稳定。

const SETTINGS_PS1: &str = include_str!("../assets/settings.ps1");

/// 弹出设置对话框,返回更新后的 config(用户取消则返回原 config)
pub fn show(config: crate::config::AppConfig) -> crate::config::AppConfig {
    let result = std::panic::catch_unwind(|| show_inner());
    if let Err(e) = result {
        crate::main_log(&format!("设置对话框启动失败,改用 notepad: {:?}", e));
        let _ = std::process::Command::new("notepad.exe")
            .arg(crate::config::AppConfig::config_file())
            .spawn();
        return config;
    }
    // PowerShell 写完 config.json,重新读
    crate::config::AppConfig::load()
}

fn show_inner() {
    // 1) 把 .ps1 写到临时文件
    //    必须带 UTF-8 BOM —— PowerShell 5.1 在中文系统默认按 GBK 读无 BOM 文件,中文乱码
    //    注意:include_str! 读入的字节如果源文件有 BOM,要先去掉再加一次
    let temp_dir = std::env::temp_dir();
    let ps1_path = temp_dir.join("token_usage_settings.ps1");
    let mut body = SETTINGS_PS1.as_bytes();
    if body.starts_with(&[0xEF, 0xBB, 0xBF]) {
        body = &body[3..]; // 去掉源 BOM
    }
    let mut content = Vec::with_capacity(body.len() + 3);
    content.extend_from_slice(&[0xEF, 0xBB, 0xBF]); // 单 BOM
    content.extend_from_slice(body);
    if let Err(e) = std::fs::write(&ps1_path, &content) {
        crate::main_log(&format!("写临时 ps1 失败: {}", e));
        return;
    }

    crate::main_log("启动 PowerShell 设置对话框...");

    // 2) 启动 PowerShell(同步等待对话框关闭)
    let status = std::process::Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-WindowStyle")
        .arg("Hidden")  // 隐藏 PowerShell console 窗口
        .arg("-File")
        .arg(&ps1_path)
        .status();

    match status {
        Ok(s) if s.success() => {
            crate::main_log("设置对话框正常关闭");
        }
        Ok(s) => {
            crate::main_log(&format!("设置对话框退出码: {:?}", s.code()));
        }
        Err(e) => {
            crate::main_log(&format!("启动 PowerShell 失败: {}", e));
        }
    }

    // 3) 清理临时文件
    let _ = std::fs::remove_file(&ps1_path);
}
