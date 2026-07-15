# token usage (source)

> 源码在子目录。主 README 在仓库根目录: [../README.md](../README.md)
>
> Source code lives in this subdirectory. The main README is at the repo root: [../README.md](../README.md)

---

## 目录结构 / Directory layout

```
minimax usage/
├── src/                  # Rust 源码
│   ├── main.rs           # 入口、CLI 路由、auto-start 同步
│   ├── tray.rs           # 托盘、菜单、Polling、注册表
│   ├── providers.rs      # Provider trait + 5 个实现
│   ├── config.rs         # AppConfig + JSON 持久化
│   ├── settings_window.rs# 启动 settings.ps1
│   ├── settings.rs       # --settings 命令(legacy)
│   └── notify.rs         # PowerShell toast
├── assets/
│   └── settings.ps1      # 设置 UI(PowerShell + WinForms)
├── installer/
│   ├── token_usage.iss   # Inno Setup 脚本
│   └── dist/             # 编译产物(不提交)
├── screenshots/          # README 引用的截图
├── build.ps1             # 一键构建
├── run-console.ps1       # --console 模式
├── Cargo.toml
└── Cargo.lock
```

## 构建 / Build

看主 README 的「从源码构建」一节。
See main README → "Build from source".

## License

[MIT](../LICENSE)
