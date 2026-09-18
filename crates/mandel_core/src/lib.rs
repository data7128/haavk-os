//! # HAAVK OS · Mandel Core（曼德尔核心）
//!
//! 用户态微内核，运行于宿主操作系统（Windows / Linux / Android）之上。
//! 标语：天空属于哈夫克，新世界就在你耳边。
//!
//! 模块职责：
//! - [`config`]：加载/解析 `mandel_core.toml` 主配置
//! - [`logger`]：HAAVK 日志系统（控制台 + 文件）
//! - [`node`]：哈夫克节点标识管理
//! - [`hardware`]：读取宿主 DMI/SMBIOS 硬件信息
//! - [`relink`]：Relink 同频 IPC 总线（骨架）
//! - [`vfs`]：曼德尔虚拟文件系统（骨架）
//!
//! 注意：曼德尔核心**不直接操作硬件**，网卡/磁盘/内存访问全部委托宿主内核。

pub mod apps;
pub mod config;
pub mod disk;
pub mod hardware;
pub mod logger;
pub mod node;
pub mod permissions;
pub mod relink;
pub mod vfs;

use std::path::Path;

use log::info;

use crate::config::{load_config, MandelConfig, DEFAULT_CONFIG_NAME, HAAVK_HOME};
use crate::hardware::HardwareInfo;
use crate::logger::HaavkLogger;
use crate::node::NodeInfo;
use crate::relink::RelinkBus;
use crate::vfs::Vfs;

/// 曼德尔核心主系统
pub struct MandelCore {
    pub config: MandelConfig,
    pub node: NodeInfo,
    pub hardware: HardwareInfo,
    pub relink: RelinkBus,
    pub vfs: Vfs,
    pub apps: apps::AppRegistry,
    pub disk: Option<disk::DiskImage>,
}

impl MandelCore {
    /// 启动曼德尔核心。
    ///
    /// - `config_path`：`Some(path)` 使用指定配置；`None` 使用默认路径
    ///   `$HAAVK_HOME/config/mandel_core.toml`（默认 `~/.haavk/...`）。
    pub fn boot(config_path: Option<&Path>) -> Result<Self, String> {
        let config = load_config(config_path).map_err(|e| format!("配置加载失败: {e}"))?;

        // 初始化日志（尽早，以便后续日志可追踪）
        HaavkLogger::init(&config.core.log_level).map_err(|e| format!("日志初始化失败: {e}"))?;

        info!("==================================================");
        info!(" HAAVK OS · Mandel Core v{} 启动", config.core.version);
        info!(" 标语：天空属于哈夫克，新世界就在你耳边");
        info!("==================================================");

        // 节点信息
        let node = NodeInfo::load(&config)?;
        info!("节点 ID：{}", node.node_id);

        // 硬件信息
        let hardware = if config.hardware.read_dmi {
            hardware::collect(&config)
        } else {
            HardwareInfo::unknown()
        };
        info!("硬件：{} / CPU {} / RAM {} MB", hardware.product_name, hardware.cpu, hardware.memory_mb);

        // Relink 总线（真实 UDP 42069 节点发现）
        let mut relink = RelinkBus::new(&config);
        if relink.enabled {
            relink.start(
                &node.node_id,
                "HAAVK全域算力终端",
                &config.core.version,
            );
        }

        // VFS（骨架）
        let mut vfs = Vfs::new(&config);
        if config.vfs.enable {
            vfs.auto_mount_host();
            info!("VFS 挂载点：{}", vfs.summary());
        }

        // 应用仓库（.hvk 应用扫描 + 内置应用注册）
        let mut app_registry = apps::AppRegistry::new();
        app_registry.scan();
        app_registry.register_builtin("haavk.files", "全域节点", &["filesystem.read"]);
        app_registry.register_builtin("haavk.settings", "系统设置", &["hardware.read"]);
        app_registry.register_builtin("haavk.about", "关于HAAVK", &[]);
        app_registry.register_builtin("haavk.terminal", "终端", &["shell.exec"]);
        info!(".hvk 应用仓库就绪：{} 个应用", app_registry.count());

        // .mandel 虚拟磁盘镜像（Stage 4）
        let img_path = std::path::PathBuf::from(&config.vfs.image_path);
        let disk = if img_path.exists() {
            disk::DiskImage::load(&img_path).ok()
        } else {
            disk::init_image(&img_path, config.hardware.storage_quota_mb).ok();
            disk::DiskImage::load(&img_path).ok()
        };
        if let Some(ref d) = disk {
            info!(".mandel 虚拟磁盘挂载：{} MB（已用 {} MB）", d.capacity_mb(), d.used_mb());
        }

        info!("曼德尔核心就绪：daemon={} app_limit={}", config.core.daemon, config.core.max_app_count);

        Ok(Self { config, node, hardware, relink, vfs, apps: app_registry, disk })
    }

    /// 生成「关于全域算力节点」面板文本。
    pub fn about_text(&self) -> String {
        let h = &self.hardware;
        format!(
            "HAAVK OS\n\
             Mandel Core：{}\n\
             节点 ID：{}\n\
             硬件：{}\n\
             CPU：{}\n\
             内存：{} MB（上限 {} MB）\n\
             存储配额：{} MB\n\
             网卡：{}\n\
             Relink：{}\n\
             标语：天空属于哈夫克，新世界就在你耳边",
            self.config.core.version,
            self.node.node_id,
            h.product_name,
            h.cpu,
            h.memory_mb,
            self.config.hardware.memory_limit_mb,
            self.config.hardware.storage_quota_mb,
            h.network,
            if self.relink.enabled { format!("同频在线:{}", self.relink.discovery_port) } else { "未启用".into() },
        )
    }

    /// 断开 HAAVK 环境（仅退出核心，不关闭宿主系统）。
    pub fn shutdown(&self) {
        info!("断开 HAAVK 环境……（宿主操作系统不受影响）");
        self.relink.stop();
    }
}

/// HAAVK 数据根目录解析（`$HAAVK_HOME` 或 `~/.haavk`）。
pub fn haavk_home() -> std::path::PathBuf {
    let home = std::env::var("HAAVK_HOME").ok().map(std::path::PathBuf::from).unwrap_or_else(|| {
        let mut p = dirs_home().unwrap_or_else(|| std::path::PathBuf::from("."));
        p.push(HAAVK_HOME);
        p
    });
    home
}

/// 跨平台获取用户主目录。
fn dirs_home() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("USERPROFILE").map(std::path::PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var_os("HOME").map(std::path::PathBuf::from)
    }
}

/// 核心默认数据目录（`~/.haavk`），用于 logs / config / about。
pub fn core_dir() -> std::path::PathBuf {
    haavk_home()
}

#[allow(dead_code)]
fn _unused() -> &'static str {
    DEFAULT_CONFIG_NAME
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn haavk_home_returns_dir() {
        let p = haavk_home();
        assert!(!p.as_os_str().is_empty());
    }
}
