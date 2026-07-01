// 设置面板:打开 JSON 配置文件让用户手动编辑
// v1 简化方案,后续可升级为 Win32 Dialog 窗口

use crate::config::AppConfig;
use anyhow::Result;

pub fn open_in_editor() -> Result<()> {
    let path = AppConfig::config_file();
    if !path.exists() {
        // 不存在则创建默认
        let default = AppConfig::default();
        default.save()?;
    }
    // 用 notepad 打开(JSON 友好)
    let _ = std::process::Command::new("notepad.exe")
        .arg(&path)
        .spawn()
        .map_err(|e| anyhow::anyhow!("打开 notepad 失败: {}", e))?;
    Ok(())
}

pub fn show_status(_config: &AppConfig) {
    // 打印当前配置到 stdout,主要用于 --console 模式
    println!("{:#?}", _config);
}