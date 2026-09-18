//! HAAVK 权限系统（Stage 3）。
//!
//! 三级权限模型：
//! - `restricted`（受限，默认）：只能访问 haavk:// 沙箱内文件，无网络
//! - `standard`（标准）：可读宿主文件、可发送 Relink 消息
//! - `trusted`（可信）：完整访问、可监听端口、可执行命令
//!
//! 权限检查：`PermissionSet::allows(&[需要的权限]) -> bool`

use std::collections::HashSet;

/// 应用权限等级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionLevel {
    Restricted,
    Standard,
    Trusted,
}

impl PermissionLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            PermissionLevel::Restricted => "restricted",
            PermissionLevel::Standard => "standard",
            PermissionLevel::Trusted => "trusted",
        }
    }
}

/// 一个应用的权限集合
#[derive(Debug, Clone)]
pub struct PermissionSet {
    pub level: PermissionLevel,
    granted: HashSet<String>,
}

impl PermissionSet {
    pub fn new(level: PermissionLevel) -> Self {
        Self { level, granted: HashSet::new() }
    }

    /// 授予一项权限
    pub fn grant(&mut self, perm: &str) {
        self.granted.insert(perm.to_string());
    }

    /// 检查是否拥有某权限（等级自动授予基础权限）
    pub fn allows(&self, required: &str) -> bool {
        // 等级豁免：trusted 放行一切；standard 放行读写类
        match self.level {
            PermissionLevel::Trusted => true,
            PermissionLevel::Standard => match required {
                "filesystem.read" | "relink.send" | "relink.receive" => true,
                other => self.granted.contains(other),
            },
            PermissionLevel::Restricted => self.granted.contains(required),
        }
    }

    /// 检查一组权限全部满足
    pub fn allows_all(&self, required: &[&str]) -> bool {
        required.iter().all(|p| self.allows(p))
    }

    pub fn granted_list(&self) -> Vec<&String> {
        self.granted.iter().collect()
    }
}

impl Default for PermissionSet {
    fn default() -> Self {
        Self::new(PermissionLevel::Restricted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trusted_allows_everything() {
        let p = PermissionSet::new(PermissionLevel::Trusted);
        assert!(p.allows("shell.exec"));
        assert!(p.allows("network.listen"));
    }

    #[test]
    fn standard_allows_basics() {
        let p = PermissionSet::new(PermissionLevel::Standard);
        assert!(p.allows("filesystem.read"));
        assert!(p.allows("relink.send"));
        assert!(!p.allows("shell.exec"));
    }

    #[test]
    fn restricted_needs_grant() {
        let p = PermissionSet::new(PermissionLevel::Restricted);
        assert!(!p.allows("filesystem.read"));
        let mut p = p;
        p.grant("filesystem.read");
        assert!(p.allows("filesystem.read"));
    }
}
