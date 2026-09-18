//! 曼德尔虚拟文件系统 VFS（Stage 1 骨架）。
//!
//! 虚拟路径前缀 `haavk://`；宿主真实目录作为挂载点映射进 HAAVK 环境。
//! Stage 1：仅建立挂载表与路径解析；Stage 2 起对接文件管理器。

use std::path::{Path, PathBuf};

use log::info;

use crate::config::MandelConfig;
use crate::haavk_home;

/// 一个挂载点：虚拟路径 → 宿主真实路径
#[derive(Debug, Clone)]
pub struct MountPoint {
    pub virtual_path: String,
    pub host_path: PathBuf,
}

pub struct Vfs {
    root_prefix: String,
    mounts: Vec<MountPoint>,
}

impl Vfs {
    pub fn new(config: &MandelConfig) -> Self {
        Self {
            root_prefix: config.vfs.root_prefix.clone(),
            mounts: Vec::new(),
        }
    }

    /// 自动挂载宿主用户目录（文档/下载/图片 等）
    pub fn auto_mount_host(&mut self) {
        let home = haavk_home();
        let user_home = user_home_dir();

        // 核心数据目录（配置/日志/关于）
        self.mount("system", home.clone());
        // 用户目录
        self.mount("home", user_home.clone());

        // 常见宿主目录（存在才挂载）
        for (name, sub) in [
            ("documents", "Documents"),
            ("downloads", "Downloads"),
            ("pictures", "Pictures"),
        ] {
            let p = user_home.join(sub);
            if p.exists() {
                self.mount(name, p);
            }
        }

        info!("VFS 自动挂载完成（{} 个挂载点）", self.mounts.len());
    }

    fn mount(&mut self, virtual_name: &str, host_path: PathBuf) {
        let vpath = format!("{}{}", self.root_prefix, virtual_name);
        self.mounts.push(MountPoint { virtual_path: vpath, host_path });
    }

    /// 解析虚拟路径 → 宿主真实路径
    pub fn resolve(&self, virtual_path: &str) -> Option<PathBuf> {
        let stripped = virtual_path.strip_prefix(&self.root_prefix)?;
        let (name, rest) = stripped.split_once('/').unwrap_or((stripped, ""));
        let m = self.mounts.iter().find(|m| m.virtual_path == format!("{}{}", self.root_prefix, name))?;
        let mut p = m.host_path.clone();
        if !rest.is_empty() {
            p.push(rest);
        }
        Some(p)
    }

    pub fn mounts(&self) -> &[MountPoint] {
        &self.mounts
    }

    pub fn root_prefix(&self) -> &str {
        &self.root_prefix
    }

    pub fn summary(&self) -> String {
        self.mounts
            .iter()
            .map(|m| format!("{} → {}", m.virtual_path, m.host_path.display()))
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

/// 获取宿主用户主目录（跨平台）
fn user_home_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
    }
}

#[allow(dead_code)]
fn _path_exists(p: &Path) -> bool {
    p.exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_virtual_path() {
        let cfg = MandelConfig::default();
        let mut vfs = Vfs::new(&cfg);
        // 手动挂载一个已知目录
        vfs.mounts.push(MountPoint {
            virtual_path: "haavk://test".into(),
            host_path: PathBuf::from("/tmp"),
        });
        let resolved = vfs.resolve("haavk://test/some/file.txt");
        assert_eq!(resolved, Some(PathBuf::from("/tmp/some/file.txt")));
        assert_eq!(vfs.resolve("haavk://other/x"), None);
    }

    #[test]
    fn root_prefix_default() {
        let cfg = MandelConfig::default();
        let vfs = Vfs::new(&cfg);
        assert_eq!(vfs.root_prefix(), "haavk://");
    }
}
