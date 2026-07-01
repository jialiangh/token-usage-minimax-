# token usage - 设置对话框 (重写版 v2)
# 关键改动:
#   1. 不用 GetNewClosure 闭包(用 $global: 显式变量,稳)
#   2. 不用 -NoNewline + Out-File(有 BOM 问题),改用 [IO.File]::WriteAllText 显式 UTF-8 无 BOM
#   3. 不依赖 "Microsoft YaHei UI" 字体(用系统 Segoe UI,布局不会算崩)
#   4. 全部 try/catch 兜底,任何异常都写到 $env:TEMP\token_usage_settings_error.log
#   5. button 用 .Tag 存 id,不再用坐标匹配

$ErrorActionPreference = 'Stop'
$logFile = Join-Path $env:TEMP 'token_usage_settings_error.log'

function Write-Log($msg) {
    try {
        # 用 ASCII 编码(437)避免 GBK 乱码
        $ascii = New-Object System.Text.ASCIIEncoding
        $line = "[{0}] {1}" -f (Get-Date -Format 'HH:mm:ss'), $msg
        [System.IO.File]::AppendAllText($logFile, ($line + "`r`n"), $ascii)
    } catch {}
}

try {
    Write-Log "=== settings.ps1 启动 ==="

    # 先算路径,所有分支都要用
    $configPath = Join-Path $env:APPDATA "token usage\config.json"
    $configDir = Split-Path $configPath -Parent

    # 支持 --auto-save <provider> <key> [<provider> <key>...] --auto-start
    # 调试模式:不弹 UI,直接写 config.json
    if ($args.Count -gt 0 -and $args[0] -eq '--self-test') {
        # 端到端测试:建窗体 + 设值 + 触发保存 + 验证
        Write-Log "self-test mode"
        Add-Type -AssemblyName System.Windows.Forms
        Add-Type -AssemblyName System.Drawing

        # 复用主 UI 流程建窗体(简版,不创建完整 UI)
        $form = New-Object System.Windows.Forms.Form
        $form.ShowInTaskbar = $false
        $form.WindowState = [System.Windows.Forms.FormWindowState]::Minimized
        $form.Opacity = 0
        $global:edits = @{}
        $global:checks = @{}
        foreach ($id in @('MiniMax', 'deepseek', 'qwen', 'glm', 'kimi')) {
            $edit = New-Object System.Windows.Forms.TextBox
            $edit.Text = ''
            $global:edits[$id] = $edit
            $check = New-Object System.Windows.Forms.CheckBox
            $check.Checked = $false
            $global:checks[$id] = $check
        }
        $autoCheck = New-Object System.Windows.Forms.CheckBox
        $autoCheck.Checked = $false
        $providerInfo = @(
            @{ id = "MiniMax" }, @{ id = "deepseek" }, @{ id = "qwen" }, @{ id = "glm" }, @{ id = "kimi" }
        )

        # 直接模拟保存(复用主保存逻辑)
        $global:edits["MiniMax"].Text = "self-test-MiniMax-key-123"
        $global:checks["MiniMax"].Checked = $true
        $global:edits["deepseek"].Text = "self-test-deepseek-key-456"
        $global:checks["deepseek"].Checked = $true
        $autoCheck.Checked = $true

        # 同步执行保存(不通过点击按钮)
        $newProviders = New-Object System.Collections.Generic.List[object]
        foreach ($info in $providerInfo) {
            $id = $info.id
            $newProviders.Add([PSCustomObject]@{
                id = $id
                enabled = [bool]$global:checks[$id].Checked
                api_key = [string]$global:edits[$id].Text
            })
        }
        $newCfg = [PSCustomObject]@{
            providers = $newProviders.ToArray()
            auto_start = [bool]$autoCheck.Checked
            primary_provider = "MiniMax"
        }
        $json = $newCfg | ConvertTo-Json -Depth 5
        $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
        [System.IO.File]::WriteAllText($configPath, $json, $utf8NoBom)
        $form.Dispose()
        Write-Log ("self-test saved {0} bytes" -f (Get-Item $configPath).Length)

        # 验证
        $verify = [System.IO.File]::ReadAllText($configPath, [System.Text.Encoding]::UTF8)
        $vc = $verify | ConvertFrom-Json
        $mm = $vc.providers | Where-Object { $_.id -eq "MiniMax" } | Select-Object -First 1
        $ds = $vc.providers | Where-Object { $_.id -eq "deepseek" } | Select-Object -First 1
        Write-Log ("verify: MiniMax enabled={0} key={1}" -f $mm.enabled, $mm.api_key)
        Write-Log ("verify: deepseek enabled={0} key={1}" -f $ds.enabled, $ds.api_key)
        Write-Log ("verify: auto_start={0}" -f $vc.auto_start)
        return
    }

    if ($args.Count -gt 0 -and $args[0] -eq '--auto-save') {
        Write-Log "auto-save mode"
        $autoStart = $false
        $pairs = @{}
        $i = 1
        while ($i -lt $args.Count) {
            $cur = [string]$args[$i]
            if ($cur -eq '--auto-start') { $autoStart = $true; $i++; continue }
            if ($i + 1 -ge $args.Count) { break }
            $next = [string]$args[$i + 1]
            $pairs[$cur] = $next
            $i += 2
        }
        Write-Log ("pairs count: {0}" -f $pairs.Count)
        Write-Log ("configPath = {0}" -f $configPath)
        Write-Log ("configDir exists: {0}" -f (Test-Path $configDir))
        # 确保目录存在
        if (-not (Test-Path $configDir)) {
            try {
                New-Item -ItemType Directory -Path $configDir -Force | Out-Null
                Write-Log "created configDir"
            } catch {
                Write-Log ("create configDir FAILED: {0}" -f $_.Exception.Message)
            }
        }
        $newProviders = New-Object System.Collections.Generic.List[object]
        foreach ($id in @('MiniMax', 'deepseek', 'qwen', 'glm', 'kimi')) {
            $newProviders.Add([PSCustomObject]@{
                id = $id
                enabled = $pairs.ContainsKey($id)
                api_key = if ($pairs.ContainsKey($id)) { [string]$pairs[$id] } else { '' }
            })
        }
        $newCfg = [PSCustomObject]@{
            providers = $newProviders.ToArray()
            auto_start = $autoStart
            primary_provider = 'MiniMax'
        }
        $json = $newCfg | ConvertTo-Json -Depth 5
        Write-Log ("json length: {0}" -f $json.Length)
        # 关键:用 [IO.File]::WriteAllText 显式 UTF-8 无 BOM
        $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
        try {
            [System.IO.File]::WriteAllText($configPath, $json, $utf8NoBom)
            Write-Log ("auto-save write OK ({0} bytes)" -f (Get-Item $configPath).Length)
        } catch {
            $errType = $_.Exception.GetType().FullName
            $errMsg = $_.Exception.Message
            $hResult = $_.Exception.HResult
            Write-Log ("WriteAllText FAILED type={0} hresult=0x{1:X} msg={2}" -f $errType, $hResult, $errMsg)
            # 备用方案:用 Out-File
            try {
                $json | Out-File -FilePath $configPath -Encoding utf8 -NoNewline
                Write-Log "Out-File fallback OK"
            } catch {
                Write-Log ("Out-File FAILED: {0}" -f $_.Exception.Message)
            }
        }
        return
    }

    Add-Type -AssemblyName System.Windows.Forms
    Add-Type -AssemblyName System.Drawing

    if (-not (Test-Path $configDir)) {
        New-Item -ItemType Directory -Path $configDir -Force | Out-Null
    }
    Write-Log "configPath = $configPath"

    # 加载配置
    $global:cfg = $null
    if (Test-Path $configPath) {
        try {
            $raw = [System.IO.File]::ReadAllText($configPath, [System.Text.Encoding]::UTF8)
            $global:cfg = $raw | ConvertFrom-Json
            Write-Log "加载已有配置"
        } catch {
            Write-Log "配置解析失败,使用默认: $_"
            $global:cfg = $null
        }
    }
    if (-not $global:cfg) {
        $global:cfg = [PSCustomObject]@{
            providers = @(
                [PSCustomObject]@{ id = "MiniMax"; enabled = $false; api_key = "" }
                [PSCustomObject]@{ id = "deepseek"; enabled = $false; api_key = "" }
                [PSCustomObject]@{ id = "qwen"; enabled = $false; api_key = "" }
                [PSCustomObject]@{ id = "glm"; enabled = $false; api_key = "" }
                [PSCustomObject]@{ id = "kimi"; enabled = $false; api_key = "" }
            )
            auto_start = $false
            primary_provider = "MiniMax"
        }
    }

    # 字体:用系统自带的 Segoe UI(必装),不用 Microsoft YaHei UI(可能没有)
    $font = [System.Drawing.Font]::new("Segoe UI", 10)
    $fontBold = [System.Drawing.Font]::new("Segoe UI", 11, [System.Drawing.FontStyle]::Bold)
    $fontSmall = [System.Drawing.Font]::new("Segoe UI", 9)
    $fontMono = [System.Drawing.Font]::new("Consolas", 10)

    # 配色(Win11 Fluent)
    $bg = [System.Drawing.Color]::FromArgb(32, 32, 32)
    $fg = [System.Drawing.Color]::FromArgb(240, 240, 240)
    $accent = [System.Drawing.Color]::FromArgb(0, 120, 212)
    $muted = [System.Drawing.Color]::FromArgb(150, 150, 150)
    $rowBg = [System.Drawing.Color]::FromArgb(45, 45, 45)
    $btnBg = [System.Drawing.Color]::FromArgb(60, 60, 60)

    $form = New-Object System.Windows.Forms.Form
    $form.Text = "token usage - 设置"
    $form.Size = [System.Drawing.Size]::new(640, 560)
    $form.StartPosition = "CenterScreen"
    $form.BackColor = $bg
    $form.ForeColor = $fg
    $form.Font = $font
    $form.FormBorderStyle = "FixedDialog"
    $form.MaximizeBox = $false
    $form.MinimizeBox = $false
    $form.KeyPreview = $true
    # 禁用 DPI 自动缩放,避免在 125% / 150% 缩放下控件错位
    $form.AutoScaleMode = [System.Windows.Forms.AutoScaleMode]::None
    $form.ClientSize = [System.Drawing.Size]::new(640, 560)

    # 标题
    $titleLabel = New-Object System.Windows.Forms.Label
    $titleLabel.Text = "选择要监视的 Provider,填入 API Key 后点保存"
    $titleLabel.Font = $fontBold
    $titleLabel.ForeColor = $fg
    $titleLabel.AutoSize = $true
    $titleLabel.Location = [System.Drawing.Point]::new(20, 15)
    $form.Controls.Add($titleLabel)

    # provider 列表(显示名 / 提示信息)
    $providerInfo = @(
        @{ id = "MiniMax";  display = "MiniMax";  hint = "MiniMax Coding Plan" }
        @{ id = "deepseek"; display = "DeepSeek"; hint = "API Key 形如 sk-..." }
        @{ id = "qwen";     display = "通义千问";   hint = "未实现,先占位" }
        @{ id = "glm";      display = "智谱 GLM";   hint = "未实现,先占位" }
        @{ id = "kimi";     display = "Kimi";     hint = "未实现,先占位" }
    )

    # 用 hashtable 存控件引用,避免闭包变量捕获问题
    $global:edits = @{}
    $global:checks = @{}

    $y = 55
    foreach ($info in $providerInfo) {
        $id = $info.id
        $provCfg = $global:cfg.providers | Where-Object { $_.id -eq $id } | Select-Object -First 1
        if (-not $provCfg) {
            $provCfg = [PSCustomObject]@{ id = $id; enabled = $false; api_key = "" }
        }

        # CheckBox(启用)
        $check = New-Object System.Windows.Forms.CheckBox
        $check.Text = $info.display
        $check.Checked = $provCfg.enabled
        $check.Location = [System.Drawing.Point]::new(20, $y + 3)
        $check.Size = [System.Drawing.Size]::new(130, 24)
        $check.ForeColor = $fg
        $check.BackColor = $bg
        $check.FlatStyle = "Flat"
        $form.Controls.Add($check)
        $global:checks[$id] = $check

        # TextBox(API key)
        $edit = New-Object System.Windows.Forms.TextBox
        $edit.Text = [string]$provCfg.api_key
        $edit.Location = [System.Drawing.Point]::new(160, $y)
        $edit.Size = [System.Drawing.Size]::new(320, 24)
        $edit.BackColor = $rowBg
        $edit.ForeColor = $fg
        $edit.BorderStyle = "FixedSingle"
        $edit.Font = $fontMono
        $edit.UseSystemPasswordChar = $true
        $form.Controls.Add($edit)
        $global:edits[$id] = $edit

        # "测试" 按钮 - 用 .Tag 存 id,不用闭包
        $testBtn = New-Object System.Windows.Forms.Button
        $testBtn.Text = "测试"
        $testBtn.Tag = $id
        $testBtn.Location = [System.Drawing.Point]::new(495, $y - 2)
        $testBtn.Size = [System.Drawing.Size]::new(60, 30)
        $testBtn.BackColor = $accent
        $testBtn.ForeColor = [System.Drawing.Color]::White
        $testBtn.FlatStyle = "Flat"
        $testBtn.FlatAppearance.BorderSize = 0
        $testBtn.FlatAppearance.MouseOverBackColor = [System.Drawing.Color]::FromArgb(0, 140, 232)
        $form.Controls.Add($testBtn)

        $y += 38
    }

    # 开机启动
    $autoCheck = New-Object System.Windows.Forms.CheckBox
    $autoCheck.Text = "开机自动启动 token usage"
    $autoCheck.Checked = [bool]$global:cfg.auto_start
    $autoCheck.Location = [System.Drawing.Point]::new(20, $y + 8)
    $autoCheck.Size = [System.Drawing.Size]::new(560, 24)
    $autoCheck.ForeColor = $fg
    $autoCheck.BackColor = $bg
    $autoCheck.FlatStyle = "Flat"
    $form.Controls.Add($autoCheck)

    # 结果提示
    $resultLabel = New-Object System.Windows.Forms.Label
    $resultLabel.Text = "点「测试」检查 Key 是否填入,点「保存」让后台开始获取用量"
    $resultLabel.Location = [System.Drawing.Point]::new(20, $y + 45)
    $resultLabel.Size = [System.Drawing.Size]::new(580, 24)
    $resultLabel.ForeColor = $muted
    $resultLabel.BackColor = $bg
    $resultLabel.Font = $fontSmall
    $form.Controls.Add($resultLabel)

    # 按钮行
    $saveBtn = New-Object System.Windows.Forms.Button
    $saveBtn.Text = "保存"
    $saveBtn.Location = [System.Drawing.Point]::new(400, $y + 85)
    $saveBtn.Size = [System.Drawing.Size]::new(100, 36)
    $saveBtn.BackColor = $accent
    $saveBtn.ForeColor = [System.Drawing.Color]::White
    $saveBtn.FlatStyle = "Flat"
    $saveBtn.FlatAppearance.BorderSize = 0
    $saveBtn.FlatAppearance.MouseOverBackColor = [System.Drawing.Color]::FromArgb(0, 140, 232)
    $saveBtn.Font = $fontBold
    $form.Controls.Add($saveBtn)

    $cancelBtn = New-Object System.Windows.Forms.Button
    $cancelBtn.Text = "取消"
    $cancelBtn.Location = [System.Drawing.Point]::new(510, $y + 85)
    $cancelBtn.Size = [System.Drawing.Size]::new(100, 36)
    $cancelBtn.BackColor = $btnBg
    $cancelBtn.ForeColor = $fg
    $cancelBtn.FlatStyle = "Flat"
    $cancelBtn.FlatAppearance.BorderSize = 0
    $cancelBtn.FlatAppearance.MouseOverBackColor = $rowBg
    $form.Controls.Add($cancelBtn)

    Write-Log "控件创建完成"

    # === 测试按钮 handler(真实调 API 测连通性) ===
    $testHandler = {
        param($s, $e)
        $id = [string]$s.Tag
        $key = [string]$global:edits[$id].Text
        if ([string]::IsNullOrWhiteSpace($key)) {
            $resultLabel.Text = "✗ [$id] 请先填写 API Key"
            $resultLabel.ForeColor = [System.Drawing.Color]::FromArgb(220, 80, 80)
            return
        }
        if ($id -eq 'qwen' -or $id -eq 'glm' -or $id -eq 'kimi') {
            $resultLabel.Text = "✗ [$id] 暂未实现,等下个版本"
            $resultLabel.ForeColor = [System.Drawing.Color]::FromArgb(220, 80, 80)
            return
        }
        $resultLabel.Text = "⏳ 正在测试 [$id] ..."
        $resultLabel.ForeColor = [System.Drawing.Color]::FromArgb(200, 180, 80)
        $resultLabel.Refresh()
        try {
            [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls11
        } catch {}
        $headers = @{ "Authorization" = "Bearer $key"; "Content-Type" = "application/json" }
        $resultMsg = ""
        try {
            switch ($id) {
                'MiniMax' {
                    $resp = Invoke-RestMethod -Uri 'https://www.minimaxi.com/v1/api/openplatform/coding_plan/remains' -Headers $headers -Method Get -TimeoutSec 10
                    if ($resp.base_resp.status_code -eq 0) {
                        $m = $resp.model_remains | Where-Object { $_.model_name -eq 'general' } | Select-Object -First 1
                        if ($m) {
                            $p5 = $m.current_interval_remaining_percent
                            $pw = $m.current_weekly_remaining_percent
                            $h5 = [int]($m.remains_time / 3600000)
                            $m5 = [int](($m.remains_time % 3600000) / 60000)
                            $resultMsg = "✓ [$id] API 通!5h 剩余 $p5% · 周剩余 $pw% · $h5 小时 $m5 分钟后重置"
                        } else {
                            $resultMsg = "✓ [$id] API 通,但找不到 general 模型数据"
                        }
                    } else {
                        $resultMsg = "✗ [$id] Key 无效: $($resp.base_resp.status_msg)"
                    }
                }
                'deepseek' {
                    $resp = Invoke-RestMethod -Uri 'https://api.deepseek.com/user/balance' -Headers $headers -Method Get -TimeoutSec 10
                    if ($resp.is_available) {
                        $b = $resp.balance_infos | Select-Object -First 1
                        $resultMsg = "✓ [$id] API 通!余额: $($b.total_balance) $($b.currency)"
                    } else {
                        $resultMsg = "✗ [$id] 账户不可用"
                    }
                }
                default {
                    $resultMsg = "✗ [$id] 未知 provider"
                }
            }
        } catch {
            $statusCode = $null
            $body = ""
            try {
                if ($_.Exception.Response) {
                    $statusCode = [int]$_.Exception.Response.StatusCode
                    $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
                    $body = $reader.ReadToEnd()
                }
            } catch {}
            if ($statusCode -eq 401 -or $statusCode -eq 403) {
                $resultMsg = "✗ [$id] Key 鉴权失败(HTTP $statusCode),请检查 API Key"
            } elseif ($statusCode) {
                $resultMsg = "✗ [$id] HTTP $statusCode · $body"
            } elseif ($_.Exception.InnerException) {
                $resultMsg = "✗ [$id] 网络错误: $($_.Exception.InnerException.Message)"
            } else {
                $resultMsg = "✗ [$id] 请求失败: $($_.Exception.Message)"
            }
        }
        $resultLabel.Text = $resultMsg
        if ($resultMsg.StartsWith('✓')) {
            $resultLabel.ForeColor = [System.Drawing.Color]::FromArgb(80, 200, 120)
        } else {
            $resultLabel.ForeColor = [System.Drawing.Color]::FromArgb(220, 80, 80)
        }
    }
    foreach ($info in $providerInfo) {
        $btn = $form.Controls | Where-Object { $_ -is [System.Windows.Forms.Button] -and $_.Tag -eq $info.id }
        if ($btn) {
            $btn.Add_Click($testHandler)
        }
    }

    # === 保存按钮 handler ===
    $saveHandler = {
        param($s, $e)
        try {
            Write-Log "save clicked"
            $newProviders = New-Object System.Collections.Generic.List[object]
            foreach ($info in $providerInfo) {
                $id = $info.id
                $newProviders.Add([PSCustomObject]@{
                    id = $id
                    enabled = [bool]$global:checks[$id].Checked
                    api_key = [string]$global:edits[$id].Text
                })
            }
            $newCfg = [PSCustomObject]@{
                providers = $newProviders.ToArray()
                auto_start = [bool]$autoCheck.Checked
                primary_provider = "MiniMax"
            }
            $json = $newCfg | ConvertTo-Json -Depth 5
            $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
            [System.IO.File]::WriteAllText($configPath, $json, $utf8NoBom)
            Write-Log ("config saved {0} bytes" -f (Get-Item $configPath).Length)
            $form.DialogResult = [System.Windows.Forms.DialogResult]::OK
            $form.Close()
        } catch {
            $errType = $_.Exception.GetType().FullName
            $errMsg = $_.Exception.Message
            Write-Log ("save FAILED type={0} msg={1}" -f $errType, $errMsg)
            $resultLabel.Text = "保存失败: $errMsg"
            $resultLabel.ForeColor = [System.Drawing.Color]::FromArgb(220, 80, 80)
        }
    }
    $saveBtn.Add_Click($saveHandler)

    # === 取消按钮 ===
    $cancelHandler = {
        $form.DialogResult = [System.Windows.Forms.DialogResult]::Cancel
        $form.Close()
    }
    $cancelBtn.Add_Click($cancelHandler)

    # === ESC 关闭 ===
    $keyDownHandler = {
        param($s, $e)
        if ($e.KeyCode -eq [System.Windows.Forms.Keys]::Escape) {
            $form.Close()
        }
    }
    $form.Add_KeyDown($keyDownHandler)

    Write-Log "弹窗中"
    $result = $form.ShowDialog()
    Write-Log "对话框关闭 result=$result"

} catch {
    Write-Log "顶层异常: $_"
    Write-Log $_.ScriptStackTrace
    # 写一个错误标记,让 Rust 知道出错了
    try {
        [System.IO.File]::WriteAllText((Join-Path $env:TEMP 'token_usage_settings_crashed.txt'), $_.ToString())
    } catch {}
}
