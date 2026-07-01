## 中文

一个轻量级的 Windows 系统托盘应用,实时监控多个 AI 服务提供商的用量配额
(MiniMax Coding Plan、DeepSeek,Qwen / GLM / Kimi 占位待接入)。

- **极小体积**:Rust 编译产物单文件 ~1.7 MB,**零外部运行时依赖**。
- **极低占用**:常驻约 10 MB 内存,15 秒一次后台轮询。
- **多 Provider**:可独立启用 / 配置每个提供商的 API Key。
- **实时托盘**:图标颜色根据剩余用量自动变化(绿 / 黄 / 红 / 灰),tooltip 显示 5 小时窗口和本周窗口百分比以及重置倒计时。
- **真实 API 测试**:设置对话框里的「测试」按钮会**真实调用**对应 Provider 的 API,返回成功信息或具体 HTTP 错误码。
- **开机自启**:可选注册表 Run 项。
- **卸载干净**:Inno Setup 标准安装包,卸载器保留用户配置。

### 截图

**设置对话框**(深色 Fluent 风格,带真实 API 测试):

![设置窗口](minimax%20usage/screenshots/settings-window.png)

**Windows 11 系统托盘 tooltip**:

![托盘 tooltip](minimax%20usage/screenshots/tray-tooltip.png)

### 下载

[Releases](../../releases) 页面提供两个 artifact:

| 文件 | 大小 | 说明 |
|---|---|---|
| `token_usage_setup_1.0.0.exe` | ~2.7 MB | **Inno Setup 安装包** — 可选安装路径,注册卸载器,可选开机自启。推荐普通用户使用。 |
| `token_usage.exe` | ~1.7 MB | 独立单文件二进制,放到任何 PATH 目录即可。 |

两个文件都针对 `x86_64-pc-windows-msvc`(Windows 10 / 11)。二进制完全自包含。

### 快速开始

1. 在最新 Release 下载 `token_usage_setup_1.0.0.exe` 并安装。
2. 安装后 **token usage** 会自动启动并在通知区域放一个图标。
3. **Windows 11 默认会把图标藏在 ^ (显示隐藏的图标) 里** —— 右键点 ^ 把 token usage 拖出来,或者直接点托盘的 **Win11 怎么找到托盘?** 菜单项。
4. 右键托盘 → **设置(API Key)...** → 填入 provider key → 点 **测试** 验证 → **保存**。

配置文件:
```
%APPDATA%\token usage\config.json
```

日志:
```
%APPDATA%\token usage\app.log
```

卸载:设置 → 应用 → token usage → 卸载,或用开始菜单快捷方式。

### 命令行

```
token_usage.exe             # 启动托盘(默认)
token_usage.exe --console   # 拉一次用量并打印到 stdout
token_usage.exe --settings  # 用 notepad 打开配置文件
token_usage.exe --help      # 帮助
```

### 支持的 Provider

| Provider | 状态 | API endpoint |
|---|---|---|
| MiniMax (MiniMax Coding Plan) | ✅ 已实现 | `https://www.minimaxi.com/v1/api/openplatform/coding_plan/remains` |
| DeepSeek | ✅ 已实现 | `https://api.deepseek.com/user/balance` |
| 通义千问 Qwen | 🚧 占位 | v1.1 接入 |
| 智谱 GLM | 🚧 占位 | v1.1 接入 |
| Kimi / Moonshot | 🚧 占位 | v1.1 接入 |

### 架构

- **Rust 单二进制**,`#![windows_subsystem = "windows"]`,双击不弹控制台。
- **托盘图标** 运行时动态绘制 16×16 RGBA 圆点(绿 / 黄 / 红 / 灰)。
- **Polling 线程** 后台每 15 秒拉一次 provider API,主线程 `PeekMessageW` + `DispatchMessageW` pump Win32 消息(`tray-icon` crate 需要外部 message pump)。
- **设置 UI** 嵌在 `assets/settings.ps1`,启动时写到 `%TEMP%`,通过 `powershell.exe -WindowStyle Hidden` 调用。脚本用 `System.Windows.Forms` 渲染深色 Fluent 风格窗口(不需要手写 Win32 GUI)。
- **通知** 同样用 PowerShell + `[System.Windows.Forms.NotifyIcon]` 实现(Win10/11 自动转 toast)。
- **持久化**:JSON 配置在 `%APPDATA%\token usage\`,用 `serde_json` 解析。

### 从源码构建

依赖:

- Rust stable toolchain (1.77+)
- Windows: Visual Studio Build Tools (MSVC) 或 MinGW-w64
- [Inno Setup 6+](https://jrsoftware.org/isdownload.php)(仅构建安装包需要)

```powershell
# 构建二进制 + 安装包
.\build.ps1

# 用 console 模式跑一次(不启动托盘)
.\run-console.ps1
```

如果你只想要二进制:

```powershell
cargo build --release
.\target\release\token_usage.exe
```

### 许可证

[MIT](LICENSE)
