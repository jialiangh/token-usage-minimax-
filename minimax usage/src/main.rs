// 关键:声明为 Windows GUI 应用,不弹控制台窗口
#![windows_subsystem = "windows"]

mod config;
mod providers;
mod tray;
mod settings;
mod settings_window;
mod notify;

use crate::config::AppConfig;
use crate::providers::fetch_all;
use std::io::Write;

fn main() -> anyhow::Result<()> {
    // 兜底:任何 panic 都先写日志再退出,方便定位崩溃点
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("!!! PANIC: {}\nbacktrace:\n{:?}", info, std::backtrace::Backtrace::capture());
        main_log(&msg);
        eprintln!("{}", msg);
    }));

    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--console" || a == "-c") {
        return run_console();
    }
    if args.iter().any(|a| a == "--settings" || a == "-s") {
        return settings::open_in_editor();
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        // GUI 模式下 --help 也写日志,不弹窗
        log_line("token usage - 多 provider 用量监视器\n用法:\n  token_usage.exe             启动托盘(默认)\n  token_usage.exe --console   拉一次数据并打印到 stdout\n  token_usage.exe --settings  打开配置文件(notepad)\n  token_usage.exe --help      显示此帮助");
        return Ok(());
    }

    // 默认:启动托盘(GUI 模式,不弹控制台)
    let cfg = AppConfig::load();
    log_line(&format!("启动 token usage v1.0.0,加载 {} providers", cfg.providers.len()));

    // 首次启动提示:告诉用户在哪里找托盘图标
    let marker = AppConfig::config_dir().join(".first_run_done");
    if !marker.exists() {
        crate::notify::show(
            "token usage 已启动",
            "在任务栏右下角找到 ^ (显示隐藏的图标),展开后右键 token usage 彩色圆点打开菜单。"
        );
        let _ = std::fs::create_dir_all(AppConfig::config_dir());
        let _ = std::fs::write(&marker, "1");
    }

    tray::run(cfg)
}

fn run_console() -> anyhow::Result<()> {
    let cfg = AppConfig::load();
    println!("token usage v1.0.0 - 多 provider 用量监视器");
    println!("配置: {}\n", AppConfig::config_file().display());
    let infos = fetch_all(&cfg);
    if infos.is_empty() {
        println!("(没有启用的 provider,编辑配置文件启用)");
    }
    for info in infos {
        println!("━━━ [{}] ━━━", info.provider_name);
        println!("  {}", info.summary);
        if !info.detail.is_empty() {
            for line in info.detail.lines() {
                println!("  {}", line);
            }
        }
        println!();
    }
    Ok(())
}

/// GUI 模式下日志写到 %LOCALAPPDATA%\token usage\app.log
fn log_line(msg: &str) {
    main_log(msg);
}

pub fn main_log(msg: &str) {
    let _ = (|| -> std::io::Result<()> {
        let dir = AppConfig::config_dir();
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("app.log");
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&path)?;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        writeln!(f, "[{}] {}", ts, msg)?;
        Ok(())
    })();
}