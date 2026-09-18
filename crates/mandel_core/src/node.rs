//! 哈夫克节点标识管理。
//!
//! 节点 ID 优先级：配置 `node_id` > 持久化 `node_id` 文件 > 自动生成 UUID。

use std::fs;
use std::path::PathBuf;

use log::info;
use uuid::Uuid;

use crate::config::MandelConfig;
use crate::haavk_home;

/// 节点持久化文件：`~/.haavk/node_id`
const NODE_ID_FILE: &str = "node_id";

pub struct NodeInfo {
    pub node_id: String,
}

impl NodeInfo {
    /// 加载节点信息。若配置未指定节点 ID，则读取或生成持久化 ID。
    pub fn load(config: &MandelConfig) -> Result<Self, String> {
        if !config.core.node_id.trim().is_empty() {
            return Ok(Self { node_id: config.core.node_id.trim().to_string() });
        }
        let persisted = load_persisted_id().unwrap_or_else(|| {
            let id = format!("HAAVK-NODE-{}", Uuid::new_v4().simple().to_string()[..8].to_uppercase());
            save_persisted_id(&id);
            id
        });
        Ok(Self { node_id: persisted })
    }
}

fn node_id_path() -> PathBuf {
    haavk_home().join(NODE_ID_FILE)
}

fn load_persisted_id() -> Option<String> {
    let p = node_id_path();
    if !p.exists() {
        return None;
    }
    fs::read_to_string(&p).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn save_persisted_id(id: &str) {
    let p = node_id_path();
    if let Some(dir) = p.parent() {
        let _ = fs::create_dir_all(dir);
    }
    match fs::write(&p, id) {
        Ok(_) => info!("节点 ID 已持久化：{id}"),
        Err(e) => eprintln!("[HAAVK] 节点 ID 持久化失败: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_from_config() {
        let cfg = MandelConfig::default();
        // 默认配置 node_id 为空 → 应生成持久化 ID
        let node = NodeInfo::load(&cfg).expect("节点加载失败");
        assert!(node.node_id.starts_with("HAAVK-NODE-"));
    }
}
