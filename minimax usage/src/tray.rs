use crate::config::AppConfig;
use crate::settings_window;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder, TrayIconEvent};

const ICON_SIZE: u32 = 16;
/// 轮询间隔(秒)。15 秒足够快让用户看到反馈,又不至于太耗资源。
const POLL_INTERVAL_SECS: u32 = 15;

struct AppState {
    config: AppConfig,
    infos: Vec<crate::providers::UsageInfo>,
    last_pct: Option<i32>,
    /// 上次更新时间(用于决定是否需要刷新托盘显示)
    last_update: Instant,
}

type SharedState = Arc<Mutex<AppState>>;

pub fn run(config: AppConfig) -> Result<()> {
    let infos = crate::providers::fetch_all(&config);
    let last_pct = infos.iter().filter_map(|i| i.color_pct).min();
    let state = Arc::new(Mutex::new(AppState {
        config,
        infos,
        last_pct,
        last_update: Instant::now(),
    }));

    // 菜单事件 handler(在主线程上调用,可以直接动 tray)
    let s_for_menu = state.clone();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        handle_menu_event(event.id.as_ref(), s_for_menu.clone());
    }));

    // 托盘事件 handler(同上)
    let s_for_tray = state.clone();
    TrayIconEvent::set_event_handler(Some(move |_event| {
        refresh_data_and_tray(&s_for_tray);
    }));

    // 初始菜单 + 图标
    let (initial_menu, initial_icon, initial_tooltip) = {
        let s = state.lock().unwrap();
        (build_menu(&s), make_color_icon(s.last_pct), make_tooltip(&s))
    };

    // 创建托盘(所有权保留在主线程,t 是 Cell 而不是 Mutex)
    let tray_cell = std::cell::RefCell::new(
        TrayIconBuilder::new()
            .with_icon(initial_icon)
            .with_tooltip(initial_tooltip)
            .with_menu(Box::new(initial_menu))
            .with_menu_on_left_click(false)
            .build()
            .map_err(|e| {
                crate::main_log(&format!("创建托盘失败: {}", e));
                anyhow::anyhow!("创建托盘失败: {}", e)
            })?,
    );

    crate::main_log("托盘已创建,进入主消息循环(polling + Win32 messages)");

    // 启动 polling 线程(只更新 state,不直接动 tray)
    spawn_polling_thread(state.clone());

    // 主消息循环:
    // - PeekMessageW 处理所有待处理的 Win32 消息(用户点击托盘菜单等)
    // - 每 POLL_INTERVAL_SECS 主动拉一次数据
    // - 拉完后只更新 tray 图标颜色 + tooltip,不再重建 menu
    //   (频繁 set_menu 会破坏 muda 的菜单事件路由,导致右键失效)
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, PM_REMOVE, WM_QUIT,
    };
    use windows::Win32::Foundation::HWND;

    let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
    let mut last_poll = Instant::now();
    loop {
        unsafe {
            // 非阻塞地处理所有待处理消息
            while PeekMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    crate::main_log("收到 WM_QUIT,退出");
                    return Ok(());
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // 检查 polling 线程是否更新过 state,有就刷新托盘
            // 只更新 icon 和 tooltip,绝不再 set_menu(menu 是固定的)
            let snap = state.lock().unwrap();
            if snap.last_update.elapsed() < Duration::from_secs(2) {
                let icon = make_color_icon(snap.last_pct);
                let tooltip = make_tooltip(&snap);
                if let Ok(t) = tray_cell.try_borrow() {
                    let _ = t.set_icon(Some(icon));
                    let _ = t.set_tooltip(Some(tooltip));
                }
            }
            drop(snap);

            // POLL_INTERVAL_SECS 到点就主动刷新一次
            if last_poll.elapsed() >= Duration::from_secs(POLL_INTERVAL_SECS.into()) {
                refresh_data_and_tray(&state);
                last_poll = Instant::now();
            }
        }

        // 避免 CPU 100% 占用
        thread::sleep(Duration::from_millis(200));
    }
}

fn refresh_data_and_tray(state: &SharedState) {
    let cfg = state.lock().unwrap().config.clone();
    let infos = crate::providers::fetch_all(&cfg);
    let last_pct = infos.iter().filter_map(|i| i.color_pct).min();
    {
        let mut s = state.lock().unwrap();
        s.infos = infos;
        s.last_pct = last_pct;
        s.last_update = Instant::now();
    }
    crate::main_log(&format!("数据刷新:{} 个 provider", state.lock().unwrap().infos.len()));
}

fn build_menu(_state: &AppState) -> Menu {
    // menu 是固定的,不再根据 state 变化重建
    // 状态信息(tray icon 颜色 + tooltip)由 set_icon/set_tooltip 更新
    let menu = Menu::new();
    let _ = menu.append(&MenuItem::with_id("refresh", "立即刷新", true, None));
    let _ = menu.append(&MenuItem::with_id("settings", "设置(API Key)...", true, None));
    let _ = menu.append(&MenuItem::with_id("auto_start", "开机启动", true, None));
    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&MenuItem::with_id("open_config", "打开配置目录", true, None));
    let _ = menu.append(&MenuItem::with_id("how_to_find", "Win11 怎么找到托盘?", true, None));
    let _ = menu.append(&MenuItem::with_id("about", "关于", true, None));
    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&MenuItem::with_id("exit", "退出", true, None));
    menu
}

fn make_tooltip(state: &AppState) -> String {
    if state.infos.is_empty() {
        return "token usage (未配置)".to_string();
    }
    state.infos.iter().map(|info| {
        // 每个 provider 一段,用 " | " 分隔
        match info.provider_name.as_str() {
            "MiniMax" => {
                let p5 = info.percent_5h.unwrap_or(0);
                let pw = info.percent_week.unwrap_or(0);
                let reset = if info.reset_5h_text.is_empty() { String::new() } else { format!(" | {}", info.reset_5h_text) };
                format!("MiniMax: 5h {}% | 周 {}%{}", p5, pw, reset)
            }
            "DeepSeek" => info.summary.clone(),
            _ => info.summary.clone(),
        }
    }).collect::<Vec<_>>().join(" | ")
}

fn handle_menu_event(id: &str, state: SharedState) {
    crate::main_log(&format!("菜单事件: {}", id));
    match id {
        "refresh" => refresh_data_and_tray(&state),
        "settings" => {
            // 弹设置窗口,完成后用新 config 替换
            let cfg = state.lock().unwrap().config.clone();
            let new_cfg = settings_window::show(cfg.clone());
            // 检测是否有改动:比对新旧 config 的 enabled + api_key
            let changed = {
                let old = &cfg;
                let new_c = &new_cfg;
                old.providers.len() != new_c.providers.len()
                    || old.providers.iter().zip(new_c.providers.iter()).any(|(o, n)| {
                        o.enabled != n.enabled || o.api_key != n.api_key
                    })
                    || old.auto_start != new_c.auto_start
            };
            {
                let mut s = state.lock().unwrap();
                s.config = new_cfg.clone();
            }
            if changed {
                crate::main_log("设置已更新,正在刷新托盘");
                crate::notify::show(
                    "token usage",
                    &format!("设置已保存 ✓\n启用 {} 个 provider,后台将立即拉取用量",
                        new_cfg.providers.iter().filter(|p| p.enabled).count())
                );
            } else {
                crate::main_log("设置未改动");
            }
            // 立即刷新托盘
            refresh_data_and_tray(&state);
        }
        "auto_start" => {
            toggle_auto_start(&state.lock().unwrap().config);
        }
        "open_config" => {
            let dir = AppConfig::config_dir();
            let _ = std::process::Command::new("explorer.exe").arg(&dir).spawn();
        }
        "how_to_find" => {
            // 弹一个 Win32 MessageBox 解释 Win11 托盘位置
            crate::notify::show("token usage", "在任务栏右下角找到 ^ (显示隐藏的图标),点开后能看到 token usage 彩色圆点图标,右键打开菜单");
        }
        "about" => {
            crate::notify::show(
                "token usage v1.0.0",
                "多 provider AI 用量监视器(MiniMax / DeepSeek ...)\n右键托盘图标打开菜单"
            );
            crate::main_log(&format!(
                "token usage v1.0.0\n程序: {}\n配置: {}",
                std::env::current_exe().unwrap_or_default().display(),
                AppConfig::config_dir().display()
            ));
        }
        "exit" => {
            crate::main_log("用户选择退出");
            std::process::exit(0);
        }
        _ => {}
    }
}

fn toggle_auto_start(config: &AppConfig) {
    let exe = std::env::current_exe().unwrap_or_default();
    let result = set_run_registry(&exe.to_string_lossy(), !config.auto_start);
    match result {
        Ok(enabled) => {
            let mut c = config.clone();
            c.auto_start = enabled;
            let _ = c.save();
            crate::main_log(&format!("开机启动: {}", if enabled { "已启用" } else { "已禁用" }));
        }
        Err(e) => crate::main_log(&format!("设置开机启动失败: {}", e)),
    }
}

pub fn set_run_registry(exe_path: &str, enable: bool) -> Result<bool> {
    use windows::Win32::System::Registry::{
        RegOpenKeyExW, RegSetValueExW, RegDeleteValueW, RegCloseKey, HKEY_CURRENT_USER,
    };
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::core::PCWSTR;

    let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0"
        .encode_utf16().collect();
    let value_name: Vec<u16> = "token_usage\0".encode_utf16().collect();

    unsafe {
        let mut hkey = windows::Win32::System::Registry::HKEY::default();
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            0,
            windows::Win32::System::Registry::KEY_SET_VALUE | windows::Win32::System::Registry::KEY_QUERY_VALUE,
            &mut hkey,
        );
        if result != ERROR_SUCCESS {
            return Err(anyhow::anyhow!("打开 Run 注册表失败: {}", result.0));
        }

        if enable {
            let data: Vec<u16> = format!("\"{}\"", exe_path).encode_utf16().chain(std::iter::once(0)).collect();
            let bytes: &[u8] = std::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                (data.len() - 1) * 2,
            );
            let result = RegSetValueExW(
                hkey,
                PCWSTR(value_name.as_ptr()),
                0,
                windows::Win32::System::Registry::REG_SZ,
                Some(bytes),
            );
            let _ = RegCloseKey(hkey);
            if result != ERROR_SUCCESS {
                return Err(anyhow::anyhow!("写注册表失败: {}", result.0));
            }
            Ok(true)
        } else {
            let result = RegDeleteValueW(hkey, PCWSTR(value_name.as_ptr()));
            let _ = RegCloseKey(hkey);
            if result != ERROR_SUCCESS && result.0 != 2 {
                return Err(anyhow::anyhow!("删注册表失败: {}", result.0));
            }
            Ok(false)
        }
    }
}

fn spawn_polling_thread(state: SharedState) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS.into()));
        // 只更新 state,不碰 tray(tray 在主线程里更新)
        let cfg = state.lock().unwrap().config.clone();
        let infos = crate::providers::fetch_all(&cfg);
        let last_pct = infos.iter().filter_map(|i| i.color_pct).min();
        let mut s = state.lock().unwrap();
        s.infos = infos;
        s.last_pct = last_pct;
        s.last_update = Instant::now();
        crate::main_log(&format!("polling 线程更新数据:{} provider", s.infos.len()));
    });
}

fn make_color_icon(pct: Option<i32>) -> Icon {
    let color = match pct {
        None => (128, 128, 128),
        Some(p) if p > 50 => (46, 204, 113),
        Some(p) if p > 20 => (241, 196, 15),
        Some(_) => (231, 76, 60),
    };

    let (r, g, b) = color;
    let mut rgba = Vec::with_capacity((ICON_SIZE * ICON_SIZE * 4) as usize);
    for y in 0..ICON_SIZE {
        for x in 0..ICON_SIZE {
            let dx = (x as f32) - (ICON_SIZE as f32 / 2.0) + 0.5;
            let dy = (y as f32) - (ICON_SIZE as f32 / 2.0) + 0.5;
            let d2 = dx * dx + dy * dy;
            if d2 < 36.0 {
                rgba.extend_from_slice(&[r, g, b, 255]);
            } else if d2 < 49.0 {
                let alpha = ((49.0 - d2) / 13.0 * 255.0) as u8;
                rgba.extend_from_slice(&[r, g, b, alpha]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }

    Icon::from_rgba(rgba, ICON_SIZE, ICON_SIZE).expect("icon")
}