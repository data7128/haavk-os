//! `.hvk` 哈夫克应用包格式（Stage 3）。
//!
//! 一个 `.hvk` 应用 = 一个目录，内含 `manifest.json`：
//! ```json
//! {
//!   "id": "haavk.files",
//!   "name": "HAAVK 文件管理器",
//!   "version": "1.0.0",
//!   "kind": "builtin",
//!   "permissions": ["filesystem.read"],
//!   "icon": "assets/haavk_256.png"
//! }
//! ```
//! 已安装应用扫描目录：`$HAAVK_HOME/apps/<id>/manifest.json`

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use log::info;
use serde::{Deserialize, Serialize};

use crate::haavk_home;

/// `.hvk` 应用清单
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppManifest {
    pub id: String,
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_kind")]
    pub kind: String, // builtin / hvk / web
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub description: String,
}

fn default_version() -> String { "1.0.0".into() }
fn default_kind() -> String { "hvk".into() }

/// 一个已安装应用
#[derive(Debug, Clone)]
pub struct InstalledApp {
    pub manifest: AppManifest,
    pub install_dir: PathBuf,
}

/// 应用仓库：扫描/枚举已安装 `.hvk` 应用
pub struct AppRegistry {
    apps: BTreeMap<String, InstalledApp>,
}

impl AppRegistry {
    pub fn new() -> Self {
        Self { apps: BTreeMap::new() }
    }

    /// 扫描 `$HAAVK_HOME/apps/` 下所有 manifest.json
    pub fn scan(&mut self) {
        self.apps.clear();
        let apps_dir = haavk_home().join("apps");
        if !apps_dir.exists() {
            let _ = fs::create_dir_all(&apps_dir);
        }
        if let Ok(entries) = fs::read_dir(&apps_dir) {
            for entry in entries.flatten() {
                let dir = entry.path();
                if !dir.is_dir() {
                    continue;
                }
                let manifest_path = dir.join("manifest.json");
                if manifest_path.exists() {
                    if let Ok(text) = fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = serde_json::from_str::<AppManifest>(&text) {
                            info!(".hvk 应用已注册：{} ({})", manifest.name, manifest.id);
                            self.apps.insert(
                                manifest.id.clone(),
                                InstalledApp { manifest, install_dir: dir },
                            );
                        }
                    }
                }
            }
        }
        info!("应用仓库扫描完成：{} 个 .hvk 应用", self.apps.len());
    }

    /// 注册一个内置应用（桌面内置，无需 manifest 文件）
    pub fn register_builtin(&mut self, id: &str, name: &str, permissions: &[&str]) {
        self.apps.insert(
            id.into(),
            InstalledApp {
                manifest: AppManifest {
                    id: id.into(),
                    name: name.into(),
                    version: "1.0.0".into(),
                    kind: "builtin".into(),
                    permissions: permissions.iter().map(|s| s.to_string()).collect(),
                    description: String::new(),
                },
                install_dir: PathBuf::new(),
            },
        );
    }

    pub fn all(&self) -> impl Iterator<Item = &InstalledApp> {
        self.apps.values()
    }

    pub fn get(&self, id: &str) -> Option<&InstalledApp> {
        self.apps.get(id)
    }

    pub fn count(&self) -> usize {
        self.apps.len()
    }

    /// 安装一个 .hvk 目录包（复制到 $HAAVK_HOME/apps/<id>/）
    pub fn install(&mut self, pkg_dir: &Path) -> Result<AppManifest, String> {
        let manifest_path = pkg_dir.join("manifest.json");
        let text = fs::read_to_string(&manifest_path).map_err(|e| format!("读取 manifest 失败: {e}"))?;
        let manifest: AppManifest =
            serde_json::from_str(&text).map_err(|e| format!("manifest 解析失败: {e}"))?;
        let target = haavk_home().join("apps").join(&manifest.id);
        let _ = fs::create_dir_all(&target);
        // 复制 manifest
        fs::copy(&manifest_path, target.join("manifest.json"))
            .map_err(|e| format!("安装失败: {e}"))?;
        info!(".hvk 应用安装完成：{} → {}", manifest.name, target.display());
        let installed = InstalledApp { manifest: manifest.clone(), install_dir: target };
        self.apps.insert(manifest.id.clone(), installed);
        Ok(manifest)
    }
}

impl Default for AppRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_parse_works() {
        let json = r#"{
            "id": "haavk.terminal",
            "name": "HAAVK 终端",
            "version": "0.2.0",
            "kind": "builtin",
            "permissions": ["shell.exec"],
            "description": "命令行终端"
        }"#;
        let m: AppManifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.id, "haavk.terminal");
        assert_eq!(m.version, "0.2.0");
        assert!(m.permissions.contains(&"shell.exec".to_string()));
    }

    #[test]
    fn register_builtin_works() {
        let mut reg = AppRegistry::new();
        reg.register_builtin("haavk.files", "全域节点", &["filesystem.read"]);
        assert_eq!(reg.count(), 1);
        assert!(reg.get("haavk.files").is_some());
        assert!(reg.get("haavk.nope").is_none());
    }
}
