//! 配置加载与解析：`mandel_core.toml`

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::haavk_home;

pub const HAAVK_HOME: &str = ".haavk";
pub const DEFAULT_CONFIG_NAME: &str = "mandel_core.toml";

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct MandelConfig {
    pub core: CoreConfig,
    pub hardware: HardwareConfig,
    pub relink_bus: RelinkConfig,
    pub vfs: VfsConfig,
    pub window_manager: WindowManagerConfig,
    pub security: SecurityConfig,
    pub network: NetworkConfig,
}

impl Default for MandelConfig {
    fn default() -> Self {
        Self {
            core: CoreConfig::default(),
            hardware: HardwareConfig::default(),
            relink_bus: RelinkConfig::default(),
            vfs: VfsConfig::default(),
            window_manager: WindowManagerConfig::default(),
            security: SecurityConfig::default(),
            network: NetworkConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CoreConfig {
    pub version: String,
    pub node_id: String,
    pub log_level: String,
    pub daemon: bool,
    pub max_app_count: u32,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            version: "0.1.0-mvp".into(),
            node_id: String::new(),
            log_level: "info".into(),
            daemon: true,
            max_app_count: 64,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct HardwareConfig {
    pub read_dmi: bool,
    pub dmi_timeout: u64,
    pub memory_limit_mb: u64,
    pub storage_quota_mb: u64,
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            read_dmi: true,
            dmi_timeout: 1000,
            memory_limit_mb: 16384,
            storage_quota_mb: 102400,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct RelinkConfig {
    pub enable: bool,
    pub node_discovery: bool,
    pub discovery_port: u16,
    pub relay_server_enable: bool,
    pub max_connections: u32,
}

impl Default for RelinkConfig {
    fn default() -> Self {
        Self {
            enable: true,
            node_discovery: true,
            discovery_port: 42069,
            relay_server_enable: false,
            max_connections: 32,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct VfsConfig {
    pub enable: bool,
    pub root_prefix: String,
    pub auto_mount_host: bool,
    pub isolate_mode: bool,
    pub image_path: String,
}

impl Default for VfsConfig {
    fn default() -> Self {
        Self {
            enable: true,
            root_prefix: "haavk://".into(),
            auto_mount_host: true,
            isolate_mode: false,
            image_path: "./haavk_storage.mandel".into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct WindowManagerConfig {
    pub enable: bool,
    pub max_windows: u32,
    pub alt_tab_switch: bool,
    pub animations: bool,
    pub render_backend: String,
}

impl Default for WindowManagerConfig {
    fn default() -> Self {
        Self {
            enable: true,
            max_windows: 48,
            alt_tab_switch: true,
            animations: true,
            render_backend: "auto".into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SecurityConfig {
    pub default_permission: String,
    pub allow_file_access: bool,
    pub allow_network_access: bool,
    pub allow_input_hook: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            default_permission: "restricted".into(),
            allow_file_access: true,
            allow_network_access: true,
            allow_input_hook: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    pub proxy_network: bool,
    pub internet_access: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            proxy_network: true,
            internet_access: true,
        }
    }
}

/// 默认配置路径：`$HAAVK_HOME/config/mandel_core.toml`
pub fn default_config_path() -> PathBuf {
    haavk_home().join("config").join(DEFAULT_CONFIG_NAME)
}

/// 加载配置：显式路径 > 默认路径；文件不存在时回退内置默认配置并提示。
/// 从 node_id（如 HAAVK-NODE-00001）提取数字编号
fn parse_node_num(node_id: &str) -> Option<u32> {
    node_id.rsplit('-').next()?.parse().ok()
}

pub fn load_config(explicit: Option<&Path>) -> Result<MandelConfig, String> {
    let path = match explicit {
        Some(p) => p.to_path_buf(),
        None => default_config_path(),
    };

    if path.exists() {
        let raw = fs::read_to_string(&path).map_err(|e| format!("读取 {} 失败: {e}", path.display()))?;
        let cfg: MandelConfig = toml::from_str(&raw).map_err(|e| format!("解析 {} 失败: {e}", path.display()))?;
        if cfg.core.node_id.is_empty() {
            return Err("配置中 node_id 为空，请检查 mandel_core.toml".into());
        }
        // DMI 节点编号上限 9999
        if let Some(num) = parse_node_num(&cfg.core.node_id) {
            if num > 9999 {
                return Err(format!("DMI 节点编号 {num} 超出上限 9999"));
            }
        }
        return Ok(cfg);
    }

    // 回退内置默认配置
    let cfg = MandelConfig::default();
    eprintln!(
        "[HAAVK] 未找到配置文件 {}，已使用内置默认配置",
        path.display()
    );
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_default_config_str() {
        // 用一段完整配置字符串验证反序列化
        let s = r#"
[core]
version = "0.1.0-mvp"
node_id = "HAAVK-NODE-00001"
log_level = "info"
daemon = true
max_app_count = 64

[hardware]
read_dmi = true
dmi_timeout = 1000
memory_limit_mb = 16384
storage_quota_mb = 102400

[relink_bus]
enable = true
node_discovery = true
discovery_port = 42069
relay_server_enable = false
max_connections = 32

[vfs]
enable = true
root_prefix = "haavk://"
auto_mount_host = true
isolate_mode = false
image_path = "./haavk_storage.mandel"

[window_manager]
enable = true
max_windows = 48
alt_tab_switch = true
animations = true
render_backend = "auto"

[security]
default_permission = "restricted"
allow_file_access = true
allow_network_access = true
allow_input_hook = false

[network]
proxy_network = true
internet_access = true
"#;
        let cfg: MandelConfig = toml::from_str(s).expect("配置解析失败");
        assert_eq!(cfg.core.node_id, "HAAVK-NODE-00001");
        assert_eq!(cfg.hardware.memory_limit_mb, 16384);
        assert_eq!(cfg.hardware.storage_quota_mb, 102400);
        assert_eq!(cfg.relink_bus.discovery_port, 42069);
        assert_eq!(cfg.vfs.root_prefix, "haavk://");
    }

    #[test]
    fn default_falls_back() {
        let cfg = MandelConfig::default();
        assert_eq!(cfg.core.log_level, "info");
        assert!(cfg.hardware.read_dmi);
    }
}
