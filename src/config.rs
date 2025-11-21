use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    /// 绑定地址
    pub bind: String,
    /// 端口
    pub port: u16,
    /// 缓存开关
    pub cache_enabled: bool,
    /// 缓存目录
    pub cache_dir: String,
    /// 缓存 TTL
    pub cache_ttl_secs: u64,
    /// 最大缓存总大小
    pub cache_max_mb: u64,
    /// 默认格式(仅在请求没有 format 参数时有效)，"webp" 或 "avif" 解析失败则 AVIF
    pub default_format: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AvifConfig {
    /// AVIF 质量
    pub quality: u8,
    /// AVIF 速度
    pub speed: u8,
    /// AVIF 编码线程数
    pub thread: Option<usize>,
}

impl ServerConfig {
    pub fn cache_max_bytes(&self) -> u64 {
        self.cache_max_mb * 1024 * 1024
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum BackendKind {
    #[serde(rename = "fs")]
    FileSystem { root: String },
    #[serde(rename = "s3")]
    S3 {
        bucket: String,
        region: String,
        access_key: String,
        secret_key: String,
        endpoint: Option<String>,
    },
    #[serde(rename = "http")]
    Http {
        root: String,
        unix_socket: Option<PathBuf>,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub struct BackendConfig {
    pub prefix: String,
    #[serde(flatten)]
    pub kind: BackendKind,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub backends: Vec<BackendConfig>,
    pub avif: AvifConfig,
}

impl Config {
    /// 读取配置文件
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&s)?)
    }

    /// 默认配置文件
    pub const fn template() -> &'static str {
        include_str!("../assets/example_config.toml")
    }
}
