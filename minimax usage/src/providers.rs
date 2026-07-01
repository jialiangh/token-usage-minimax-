use crate::config::AppConfig;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::time::Duration;

/// 统一用量信息(各 provider 都映射到这个结构)
#[derive(Debug, Clone)]
pub struct UsageInfo {
    pub provider_id: String,
    pub provider_name: String,
    /// 5h 窗口剩余百分比(0-100,None 表示未知)
    pub percent_5h: Option<i32>,
    /// 周窗口剩余百分比
    pub percent_week: Option<i32>,
    /// 5h 重置倒计时文案(已格式化,"4 小时 49 分钟后重置")
    pub reset_5h_text: String,
    /// 周重置倒计时文案
    pub reset_week_text: String,
    /// 短摘要(用于 tooltip)
    pub summary: String,
    /// 详细文本(用于详情窗口)
    pub detail: String,
    /// 用于决定托盘颜色的百分比(None = 灰色)
    pub color_pct: Option<i32>,
}

impl UsageInfo {
    pub fn error(provider_id: &str, provider_name: &str, msg: &str) -> Self {
        Self {
            provider_id: provider_id.into(),
            provider_name: provider_name.into(),
            percent_5h: None,
            percent_week: None,
            reset_5h_text: String::new(),
            reset_week_text: String::new(),
            summary: format!("✗ {}", msg),
            detail: format!("错误: {}", msg),
            color_pct: None,
        }
    }

    pub fn no_key(provider_id: &str, provider_name: &str) -> Self {
        Self {
            provider_id: provider_id.into(),
            provider_name: provider_name.into(),
            percent_5h: None,
            percent_week: None,
            reset_5h_text: String::new(),
            reset_week_text: String::new(),
            summary: "未配置".into(),
            detail: "请在右键菜单 → 设置 中配置 API Key".into(),
            color_pct: None,
        }
    }
}

pub trait Provider {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn fetch(&self, api_key: &str, custom_endpoint: Option<&str>) -> Result<UsageInfo>;
}

// === MiniMax ===
pub struct MiniMaxProvider;
impl Provider for MiniMaxProvider {
    fn id(&self) -> &'static str { "MiniMax" }
    fn display_name(&self) -> &'static str { "MiniMax" }
    fn fetch(&self, api_key: &str, _custom: Option<&str>) -> Result<UsageInfo> {
        #[derive(Deserialize)]
        struct ModelRemain {
            model_name: String,
            current_interval_remaining_percent: i32,
            current_weekly_remaining_percent: i32,
            end_time: i64,
            remains_time: i64,
            weekly_remains_time: i64,
        }
        #[derive(Deserialize)]
        struct Resp {
            model_remains: Vec<ModelRemain>,
            base_resp: BaseResp,
        }
        #[derive(Deserialize)]
        struct BaseResp {
            status_code: i32,
            status_msg: String,
        }

        let body: Resp = ureq::get("https://www.minimaxi.com/v1/api/openplatform/coding_plan/remains")
            .set("Authorization", &format!("Bearer {}", api_key))
            .timeout(Duration::from_secs(15))
            .call()
            .map_err(|e| anyhow!("请求失败: {}", e))?
            .into_json()
            .map_err(|e| anyhow!("解析失败: {}", e))?;

        if body.base_resp.status_code != 0 {
            return Err(anyhow!("API: {}", body.base_resp.status_msg));
        }

        let m = body.model_remains.iter()
            .find(|x| x.model_name == "general")
            .ok_or_else(|| anyhow!("无 general 模型数据"))?;

        let pct5 = m.current_interval_remaining_percent;
        let pctw = m.current_weekly_remaining_percent;
        let min_pct = pct5.min(pctw);

        let h5 = (m.remains_time / 3600000) as i32;
        let min5 = ((m.remains_time % 3600000) / 60000) as i32;
        let reset_5h = format!("{} 小时 {} 分钟后重置", h5, min5);

        let dw = (m.weekly_remains_time / 86400000) as i32;
        let hw = ((m.weekly_remains_time % 86400000) / 3600000) as i32;
        let reset_w = format!("{} 天 {} 小时", dw, hw);

        Ok(UsageInfo {
            provider_id: "MiniMax".into(),
            provider_name: "MiniMax".into(),
            percent_5h: Some(pct5),
            percent_week: Some(pctw),
            reset_5h_text: reset_5h.clone(),
            reset_week_text: reset_w.clone(),
            summary: format!("5h {}% / 周 {}% | {}", pct5, pctw, reset_5h),
            detail: format!(
                "模型: general\n\n5小时窗口: {}%   {}\n本周窗口:  {}%   {}\n\n更新于: {}",
                pct5, reset_5h, pctw, reset_w,
                chrono_like_now()
            ),
            color_pct: Some(min_pct),
        })
    }
}

// === DeepSeek ===
pub struct DeepSeekProvider;
impl Provider for DeepSeekProvider {
    fn id(&self) -> &'static str { "deepseek" }
    fn display_name(&self) -> &'static str { "DeepSeek" }
    fn fetch(&self, api_key: &str, _custom: Option<&str>) -> Result<UsageInfo> {
        #[derive(Deserialize)]
        struct BalanceInfo {
            currency: String,
            total_balance: String,
            granted_balance: Option<String>,
            topped_up_balance: Option<String>,
        }
        #[derive(Deserialize)]
        struct Resp {
            is_available: bool,
            balance_infos: Vec<BalanceInfo>,
        }

        let body: Resp = ureq::get("https://api.deepseek.com/user/balance")
            .set("Authorization", &format!("Bearer {}", api_key))
            .timeout(Duration::from_secs(15))
            .call()
            .map_err(|e| anyhow!("请求失败: {}", e))?
            .into_json()
            .map_err(|e| anyhow!("解析失败: {}", e))?;

        if !body.is_available {
            return Err(anyhow!("账户不可用"));
        }

        let b = body.balance_infos.first()
            .ok_or_else(|| anyhow!("无余额信息"))?;

        // 简单粗暴:把余额数字直接当 percent 展示(DeepSeek 不分 5h/周窗口)
        // 实际是 CNY 余额,这里当作一个数字显示,带 ¥ 前缀
        Ok(UsageInfo {
            provider_id: "deepseek".into(),
            provider_name: "DeepSeek".into(),
            percent_5h: None,
            percent_week: None,
            reset_5h_text: String::new(),
            reset_week_text: String::new(),
            summary: format!("余额: {} {}", b.total_balance, b.currency),
            detail: format!(
                "账户余额: {} {}\n  赠送: {} {}\n  充值: {} {}\n\n更新于: {}",
                b.total_balance, b.currency,
                b.granted_balance.as_deref().unwrap_or("0"), b.currency,
                b.topped_up_balance.as_deref().unwrap_or("0"), b.currency,
                chrono_like_now()
            ),
            color_pct: None,  // DeepSeek 不参与颜色判断
        })
    }
}

// 占位 provider(Qwen/GLM/Kimi),等用户填 endpoint
pub struct PlaceholderProvider { id: String, name: String }
impl Provider for PlaceholderProvider {
    fn id(&self) -> &'static str { "placeholder" }
    fn display_name(&self) -> &'static str { "placeholder" }
    fn fetch(&self, _api_key: &str, _custom: Option<&str>) -> Result<UsageInfo> {
        Err(anyhow!("该 provider 的 endpoint 暂未配置,等下个版本"))
    }
}

/// 工厂:根据 id 返回 provider
pub fn get_provider(id: &str) -> Box<dyn Provider> {
    match id {
        "MiniMax" => Box::new(MiniMaxProvider),
        "deepseek" => Box::new(DeepSeekProvider),
        "qwen" => Box::new(PlaceholderProvider { id: "qwen".into(), name: "通义千问 Qwen".into() }),
        "glm" => Box::new(PlaceholderProvider { id: "glm".into(), name: "智谱 GLM".into() }),
        "kimi" => Box::new(PlaceholderProvider { id: "kimi".into(), name: "Kimi / Moonshot".into() }),
        _ => Box::new(PlaceholderProvider { id: id.into(), name: id.into() }),
    }
}

/// 拉取所有启用 provider 的用量
pub fn fetch_all(config: &AppConfig) -> Vec<UsageInfo> {
    config.providers.iter()
        .filter(|p| p.enabled)
        .map(|p| {
            let provider = get_provider(&p.id);
            if p.api_key.trim().is_empty() {
                UsageInfo::no_key(&p.id, provider.display_name())
            } else {
                provider.fetch(&p.api_key, p.custom_endpoint.as_deref())
                    .unwrap_or_else(|e| UsageInfo::error(&p.id, provider.display_name(), &e.to_string()))
            }
        })
        .collect()
}

fn chrono_like_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}