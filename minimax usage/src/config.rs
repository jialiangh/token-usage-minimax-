use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};

/// 单个 provider 的配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,          // "MiniMax" / "deepseek" / ...
    pub enabled: bool,
    pub api_key: String,
    /// 可选的自定义 endpoint(用于未预设的 provider)
    pub custom_endpoint: Option<String>,
}

/// 应用总配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub providers: Vec<ProviderConfig>,
    pub auto_start: bool,
    /// 托盘主显示哪个 provider(默认第一个启用的)
    pub primary_provider: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            providers: vec![
                ProviderConfig {
                    id: "MiniMax".into(),
                    enabled: false,
                    api_key: String::new(),
                    custom_endpoint: None,
                },
                ProviderConfig {
                    id: "deepseek".into(),
                    enabled: false,
                    api_key: String::new(),
                    custom_endpoint: None,
                },
                ProviderConfig {
                    id: "qwen".into(),
                    enabled: false,
                    api_key: String::new(),
                    custom_endpoint: None,
                },
                ProviderConfig {
                    id: "glm".into(),
                    enabled: false,
                    api_key: String::new(),
                    custom_endpoint: None,
                },
                ProviderConfig {
                    id: "kimi".into(),
                    enabled: false,
                    api_key: String::new(),
                    custom_endpoint: None,
                },
            ],
            auto_start: false,
            primary_provider: Some("MiniMax".into()),
        }
    }
}

impl AppConfig {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::env::temp_dir())
            .join("token usage")
    }

    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_file();
        match std::fs::read_to_string(&path) {
            Ok(mut s) => {
                // 去掉可能存在的 UTF-8 BOM(serde_json 不识别)
                if s.starts_with('\u{FEFF}') {
                    s.remove(0);
                }
                serde_json::from_str(&s).unwrap_or_default()
            }
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir).context("create config dir")?;
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::config_file(), json).context("write config")?;
        Ok(())
    }

    pub fn get_provider(&self, id: &str) -> Option<&ProviderConfig> {
        self.providers.iter().find(|p| p.id == id)
    }

    pub fn get_provider_mut(&mut self, id: &str) -> Option<&mut ProviderConfig> {
        self.providers.iter_mut().find(|p| p.id == id)
    }
}