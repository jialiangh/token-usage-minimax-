# token usage

> Windows 系统托盘小工具,实时监控多个 AI 服务的用量配额。
> A lightweight Windows tray app that monitors multi-provider AI usage quotas in real time.

[![Latest Release](https://img.shields.io/github/v/release/jialiangh/token-usage-minimax-?style=flat-square)](https://github.com/jialiangh/token-usage-minimax-/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](LICENSE)
[![Platform: Windows 10/11](https://img.shields.io/badge/platform-Windows%2010%2F11-blue?style=flat-square)](#-quick-start)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange?style=flat-square)](https://www.rust-lang.org/)


---

## 📥 下载 / Download

### 一句话选哪个 / Which one should I download?

| 你的情况 / Situation | 下载这个 / Download this |
|---|---|
| 长期日常使用,想要开机自启、卸载器、开始菜单快捷方式<br/>Daily long-term use, want auto-start at logon, uninstaller, Start Menu shortcut | **👉 安装包 Installer** (推荐) |
| 只是想试一下 / 不想留痕 / 装在 U 盘<br/>Just want to try it out / leave no trace / run from a USB stick | **👉 独立 EXE Standalone** |

### v1.0.0(2026-07-15 发布)

| 文件 / File | 大小 / Size | SHA256 |
|---|---|---|
| [`token_usage_setup_v1.0.0.exe`](https://github.com/jialiangh/token-usage-minimax-/releases/download/v1.0.0/token_usage_setup_v1.0.0.exe) | 2.74 MB | `9d8cc8c6858da948f8441f29e182bac4b8bc33098e695b4660e9ad2811b92711` |
| [`token_usage_v1.0.0.exe`](https://github.com/jialiangh/token-usage-minimax-/releases/download/v1.0.0/token_usage_v1.0.0.exe) | 1.67 MB | `f0b5d58e6f425be380cbb373e6aeec1327d656eb637e6961e92b9b2d61b24d55` |

校验方法 / Verify:
```powershell
# Windows PowerShell
Get-FileHash .\token_usage_setup_v1.0.0.exe -Algorithm SHA256
Get-FileHash .\token_usage_v1.0.0.exe     -Algorithm SHA256
```
应该看到上面表格里的 hash。比较结果一致就说明文件没被篡改。

---

## 🔍 两个 EXE 的区别 / Two EXEs, what's the difference?

很多人会困惑「这俩下哪个」,这里一次性说清楚。

A common question: "which one should I download?" Here's the full breakdown.

| 维度 / Aspect | **安装包 Installer**<br/>`token_usage_setup_v1.0.0.exe` | **独立 EXE Standalone**<br/>`token_usage_v1.0.0.exe` |
|---|---|---|
| **是什么 / What it is** | Inno Setup 写的 Windows 安装程序 | 编译好的单文件 Rust 二进制,无需安装 |
| **安装位置 / Where it goes** | `C:\Program Files\token usage\`(可改) | 你想放哪就放哪(桌面、D 盘、U 盘) |
| **开始菜单 / Start Menu** | ✅ 自动创建快捷方式 | ❌ 自己建 |
| **卸载器 / Uninstaller** | ✅ 控制面板 → 应用 → token usage → 卸载 | ❌ 直接删掉 exe 即可 |
| **开机自启 / Auto-start at logon** | ✅ **安装时默认勾选**「开机启动」任务,自动写注册表 `HKCU\...\Run\token_usage` | ❌ 安装后右键托盘 → 「开机启动」自己开 |
| **开机自启卸载清理 / Auto-start cleanup on uninstall** | ✅ `Flags: uninsdeletevalue` 卸载时自动清注册表 | ❌ 删 exe 之后要自己清注册表项 |
| **配置文件 / Config** | `%APPDATA%\token usage\config.json`(同) | `%APPDATA%\token usage\config.json`(同) |
| **大小 / Size** | 2.74 MB(含卸载器 + Inno Setup 运行时) | 1.67 MB(纯二进制) |
| **多用户 / Multi-user** | 每个用户独立安装 | 共享一个 exe,每个人各自的 config |
| **携带性 / Portability** | 装过的机器 | U 盘 / 网盘带走,任意 Win10/11 机器双击即用 |
| **适合谁 / Best for** | **普通用户 / 长期使用** | **试用 / 极简 / 便携场景** |

> 💡 **建议 / Recommendation**: 第一次用、要在自己电脑上长期监控 → **安装包**。先看看效果、不想安装 → **独立 EXE**。
> First time / long-term use → installer. Try before commit / no-install needed → standalone.

---

## 🖼️ 截图 / Screenshots

**设置对话框 / Settings dialog**(深色 Fluent 风格,带真实 API 测试 / dark Fluent-style with live API test):

![Settings](minimax%20usage/screenshots/settings-window.png)

**Windows 11 系统托盘 / Tray tooltip**(图标颜色随剩余用量变化 / icon color reflects remaining quota):

![Tray tooltip](minimax%20usage/screenshots/tray-tooltip.png)

---

## ✨ 功能 / Features

- **极小体积 / Tiny binary** — Rust 编译产物单文件 ~1.7 MB,**零外部运行时依赖**。
- **极低占用 / Low overhead** — 常驻约 10 MB 内存,15 秒一次后台轮询。
- **多 Provider / Multi-provider** — MiniMax Coding Plan、DeepSeek 已上线,Qwen / GLM / Kimi 1.1 接入。
- **实时托盘 / Live tray** — 图标颜色随剩余用量自动变化(绿/黄/红/灰),tooltip 显示 5h 窗口 + 周窗口百分比 + 重置倒计时。
- **真实 API 测试 / Real API test** — 设置里点「测试」会**真打** provider API,返回 `✓ API 通!5h 剩余 X% · 周剩余 Y%` 或具体 HTTP 错误码。
- **开机自启 / Auto-start** — 可选 Run-key 注册表项,安装包默认开,独立版托盘里手动开。
- **卸载干净 / Clean uninstall** — 安装包带 Inno Setup 标准卸载器,自动清注册表。

---

## 🚀 快速开始 / Quick Start

### 方式 A:用安装包(推荐)/ Option A: Installer (recommended)

1. 下载 [`token_usage_setup_v1.0.0.exe`](https://github.com/jialiangh/token-usage-minimax-/releases/download/v1.0.0/token_usage_setup_v1.0.0.exe) → 双击 → 默认安装。
   *默认会勾选「开机启动」,可手动取消。/ The "startup" task is checked by default — uncheck if you don't want it.*
2. 安装完会自动启动,通知区域会出现一个托盘图标。
   *Win11 默认把图标藏在 `^` 箭头里 — 右键 `^` 把 token usage 拖出来,或在托盘菜单点「Win11 怎么找到托盘?」看提示。*
3. 右键托盘 → **设置(API Key)...** → 填入 provider key → 点 **测试** 验证 → **保存**。
4. 完事。Save 完会弹一个 toast 「设置已保存 ✓」。

### 方式 B:独立 EXE / Option B: Standalone EXE

1. 下载 [`token_usage_v1.0.0.exe`](https://github.com/jialiangh/token-usage-minimax-/releases/download/v1.0.0/token_usage_v1.0.0.exe) → 放到任何你想放的位置(桌面、某个目录)。
2. 双击运行。要开机自启就右键托盘 → **开机启动**(会写 `HKCU\...\Run\token_usage` 注册表项)。
3. 右键托盘 → **设置(API Key)...** → 填 key → 测试 → 保存。

### 文件位置 / File locations

```
%APPDATA%\token usage\config.json   ← 配置(config)
%APPDATA%\token usage\app.log       ← 日志(logs)
```

要卸载:
- 安装包:设置 → 应用 → 搜索 token usage → 卸载
- 独立 EXE:直接删掉 exe + 上面那个 `%APPDATA%\token usage` 目录即可

---

## 🛠️ 命令行 / Command line

```
token_usage.exe             # 启动托盘(默认)
token_usage.exe --console   # 拉一次所有启用的 provider 用量,打印到 stdout,退出
token_usage.exe --settings  # 用 notepad 打开 config.json(legacy 兜底,平时用托盘菜单就行)
token_usage.exe --help      # 帮助
```

`--console` 模式很适合放进 CI / 脚本里做监控,比如 `& token_usage.exe --console | Tee-Object -FilePath usage.log`。

---

## 🤖 支持的 Provider / Supported Providers

| Provider | 状态 / Status | API endpoint | 文档 / Docs |
|---|---|---|---|
| **MiniMax**(MiniMax Coding Plan) | ✅ 已实现 / Implemented | `https://www.minimaxi.com/v1/api/openplatform/coding_plan/remains` | MiniMax 开放平台 |
| **DeepSeek** | ✅ 已实现 / Implemented | `https://api.deepseek.com/user/balance` | DeepSeek API |
| 通义千问 Qwen | 🚧 占位 / Placeholder | (1.1 接入) | 阿里云百炼 |
| 智谱 GLM | 🚧 占位 / Placeholder | (1.1 接入) | 智谱开放平台 |
| Kimi / Moonshot | 🚧 占位 / Placeholder | (1.1 接入) | Moonshot AI |

要加新 provider?看 [`src/providers.rs`](minimax%20usage/src/providers.rs) — 30 行实现 `Provider` trait 就够了。

---

## 🏗️ 架构 / Architecture

- **Rust 单二进制** + `#![windows_subsystem = "windows"]`,双击不弹控制台。
- **托盘图标** 运行时绘制 16×16 RGBA 圆点(绿/黄/红/灰),颜色 = `min(所有启用的 provider 剩余百分比)`。
- **Polling 线程** 后台每 15 秒拉一次 provider API,**主线程** `PeekMessageW` + `DispatchMessageW` pump Win32 消息(`tray-icon` crate 不会自己 pump)。
- **设置 UI** 嵌在 `assets/settings.ps1`,启动时写到 `%TEMP%`,通过 `powershell.exe -WindowStyle Hidden` 调用。脚本用 `System.Windows.Forms` 渲染深色 Fluent 风格窗口 — 不需要手写 Win32 GUI。
- **通知** 同样用 PowerShell + `[System.Windows.Forms.NotifyIcon]`(Win10/11 自动转 toast notification)。
- **自启状态同步** — 启动时读 `HKCU\...\Run\token_usage` 注册表,与 `config.json` 对账,以**注册表**为权威。
- **持久化** — JSON config in `%APPDATA%\token usage\`,parsed by `serde_json`。

源码在 [`minimax usage/src/`](minimax%20usage/src/),看 `tray.rs` 了解主消息循环,`providers.rs` 了解 provider 协议。

---

## 🛠️ 从源码构建 / Build from source

依赖 / Prerequisites:

- **Rust** stable toolchain(1.77+,需要 `windows_subsystem` 支持)
- **Windows 10 / 11**
- Visual Studio Build Tools (MSVC) **或** MinGW-w64
- [Inno Setup 6+](https://jrsoftware.org/isdownload.php)(**仅**构建安装包需要 / only for building the installer)

一键构建 binary + installer / One-shot:

```powershell
cd "minimax usage"
.\build.ps1
```

产物:
- `target\release\token_usage.exe` — 独立 EXE
- `installer\dist\token_usage_setup_1.0.0.exe` — 安装包

只要二进制 / Just the binary:

```powershell
cd "minimax usage"
cargo build --release
.\target\release\token_usage.exe
```

Console 模式跑一次(不打托盘)/ Console mode, one-shot:

```powershell
cd "minimax usage"
.\run-console.ps1
```

---

## ❓ 常见问题 / FAQ

**Q: Win11 找不到托盘图标?**
A: Win11 默认把所有不常用的托盘图标藏在 `^` 箭头里。两种办法:
1. 右键 `^` → 找到 token usage → 拖出来
2. 托盘菜单里点 **「Win11 怎么找到托盘?」** — 会弹一个提醒 toast

**Q: API key 会被上传吗?**
A: 不会。所有 key 只存在你本地的 `%APPDATA%\token usage\config.json`,代码完全开源,你可以 `grep` 验证没任何外发逻辑。`Provider::fetch` 直接 `ureq` 打对应 API,没有中间人。

**Q: 为什么 v1.0 已经有 5 个 provider 但只实装 2 个?**
A: 想先放出框架让 MiniMax + DeepSeek 用户跑起来。剩下 3 个的 API 协议需要你贡献 — PR 欢迎。Provider trait 非常简单,30 行就能加一个。

**Q: 安装包会写 HKLM(全局注册表)吗?**
A: 不会。`PrivilegesRequired=admin` 只是为了在受限环境下也能装,实际只写 `HKCU`(当前用户),不需要管理员权限也能运行。卸载时 `uninsdeletevalue` 自动清 `HKCU\...\Run\token_usage`。

**Q: 配置文件坏了怎么办?**
A: 删 `%APPDATA%\token usage\config.json` 即可,下次启动会用默认值重建。

---

## 📜 许可证 / License

[MIT](LICENSE) — 你拿去用、改、商用都行,只要保留版权声明。
Use, modify, ship commercially — just keep the copyright notice.
