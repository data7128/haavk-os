//! `.mandel` 曼德尔虚拟磁盘镜像（Stage 4）。
//!
//! 格式：单文件镜像，超级块 + inode 表 + 数据块。
//! - 魔数：`HAAVK-MANDEL-V1`
//! - 块大小：4096 字节
//! - 最大容量：100 GB（配置 storage_quota_mb=102400）
//!
//! 本模块仅在用户态读写镜像文件，不挂载宿主块设备；
//! 隔离模式下 HAAVK 应用仅可见此镜像内的文件树。

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use log::info;
use serde::{Deserialize, Serialize};

pub const MANDEL_MAGIC: &[u8; 16] = b"HAAVK-MANDEL-V1\x00";
pub const BLOCK_SIZE: usize = 4096;

/// Inode 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InodeKind {
    File,
    Dir,
}

/// 一个 inode（文件或目录）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inode {
    pub id: u32,
    pub kind: InodeKind,
    pub name: String,
    pub parent: u32,
    pub size: u64,
    pub children: Vec<u32>,
    #[serde(default)]
    pub data: Vec<u8>,
}

/// 超级块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperBlock {
    pub magic: [u8; 16],
    pub version: u32,
    pub block_size: u32,
    pub total_blocks: u64,
    pub used_blocks: u64,
    pub root_inode: u32,
}

/// 完整磁盘镜像（可序列化到 .mandel 文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskImage {
    pub superblock: SuperBlock,
    pub inodes: BTreeMap<u32, Inode>,
    next_inode: u32,
}

impl DiskImage {
    /// 创建一个空镜像（capacity_mb 为容量）
    pub fn create(capacity_mb: u64) -> Self {
        let total_blocks = capacity_mb * 1024 * 1024 / BLOCK_SIZE as u64;
        let root = Inode {
            id: 0,
            kind: InodeKind::Dir,
            name: "/".into(),
            parent: 0,
            size: 0,
            children: Vec::new(),
            data: Vec::new(),
        };
        let mut inodes = BTreeMap::new();
        inodes.insert(0, root);
        Self {
            superblock: SuperBlock {
                magic: *MANDEL_MAGIC,
                version: 1,
                block_size: BLOCK_SIZE as u32,
                total_blocks,
                used_blocks: 1,
                root_inode: 0,
            },
            inodes,
            next_inode: 1,
        }
    }

    /// 按路径查找 inode id（如 /docs/readme.txt）
    pub fn lookup(&self, path: &str) -> Option<u32> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut cur = self.superblock.root_inode;
        for part in parts {
            let dir = self.inodes.get(&cur)?;
            let child = dir
                .children
                .iter()
                .find_map(|&cid| self.inodes.get(&cid).filter(|c| c.name == part))?;
            cur = child.id;
        }
        Some(cur)
    }

    /// 列出目录内容
    pub fn list_dir(&self, path: &str) -> Result<Vec<(&str, bool)>, String> {
        let id = self.lookup(path).ok_or_else(|| format!("路径不存在: {path}"))?;
        let dir = self.inodes.get(&id).ok_or("inode 损坏")?;
        if dir.kind != InodeKind::Dir {
            return Err(format!("不是目录: {path}"));
        }
        let mut out: Vec<(&str, bool)> = dir
            .children
            .iter()
            .filter_map(|&cid| self.inodes.get(&cid).map(|c| (c.name.as_str(), c.kind == InodeKind::Dir)))
            .collect();
        out.sort_by(|a, b| a.0.cmp(b.0));
        Ok(out)
    }

    /// 创建目录（父路径 + 目录名）
    pub fn mkdir(&mut self, parent_path: &str, name: &str) -> Result<u32, String> {
        let parent_id = self.lookup(parent_path).ok_or("父目录不存在")?;
        let id = self.next_inode;
        self.next_inode += 1;
        let inode = Inode {
            id,
            kind: InodeKind::Dir,
            name: name.into(),
            parent: parent_id,
            size: 0,
            children: Vec::new(),
            data: Vec::new(),
        };
        self.inodes.insert(id, inode);
        self.inodes.get_mut(&parent_id).unwrap().children.push(id);
        self.superblock.used_blocks += 1;
        Ok(id)
    }

    /// 创建文件并写入数据
    pub fn write_file(&mut self, parent_path: &str, name: &str, data: &[u8]) -> Result<u32, String> {
        let parent_id = self.lookup(parent_path).ok_or("父目录不存在")?;
        let id = self.next_inode;
        self.next_inode += 1;
        let blocks_used = (data.len() as u64 + BLOCK_SIZE as u64 - 1) / BLOCK_SIZE as u64;
        let inode = Inode {
            id,
            kind: InodeKind::File,
            name: name.into(),
            parent: parent_id,
            size: data.len() as u64,
            children: Vec::new(),
            data: data.to_vec(),
        };
        self.inodes.insert(id, inode);
        self.inodes.get_mut(&parent_id).unwrap().children.push(id);
        self.superblock.used_blocks += blocks_used.max(1);
        Ok(id)
    }

    /// 读取文件内容
    pub fn read_file(&self, path: &str) -> Result<&[u8], String> {
        let id = self.lookup(path).ok_or_else(|| format!("文件不存在: {path}"))?;
        let inode = self.inodes.get(&id).ok_or("inode 损坏")?;
        if inode.kind != InodeKind::File {
            return Err(format!("不是文件: {path}"));
        }
        Ok(&inode.data)
    }

    /// 镜像总大小（MB）
    pub fn capacity_mb(&self) -> u64 {
        self.superblock.total_blocks * BLOCK_SIZE as u64 / (1024 * 1024)
    }

    /// 已用大小（MB）
    pub fn used_mb(&self) -> u64 {
        self.superblock.used_blocks * BLOCK_SIZE as u64 / (1024 * 1024)
    }

    /// 保存到 .mandel 文件
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let mut f = File::create(path).map_err(|e| format!("创建镜像失败: {e}"))?;
        // 写魔数
        f.write_all(MANDEL_MAGIC).map_err(|e| e.to_string())?;
        // 序列化剩余内容（bincode 风格 JSON 前缀）
        let body = serde_json::to_vec(self).map_err(|e| format!("序列化失败: {e}"))?;
        // 写长度（小端 u64）
        f.write_all(&(body.len() as u64).to_le_bytes()).map_err(|e| e.to_string())?;
        f.write_all(&body).map_err(|e| e.to_string())?;
        info!("曼德尔镜像已保存: {}（{} MB）", path.display(), self.capacity_mb());
        Ok(())
    }

    /// 从 .mandel 文件加载
    pub fn load(path: &Path) -> Result<Self, String> {
        let mut f = File::open(path).map_err(|e| format!("打开镜像失败: {e}"))?;
        let mut magic = [0u8; 16];
        f.read_exact(&mut magic).map_err(|e| e.to_string())?;
        if &magic != MANDEL_MAGIC {
            return Err("不是 HAAVK 曼德尔镜像文件（魔数不匹配）".into());
        }
        let mut len_buf = [0u8; 8];
        f.read_exact(&mut len_buf).map_err(|e| e.to_string())?;
        let len = u64::from_le_bytes(len_buf) as usize;
        let mut body = vec![0u8; len];
        f.read_exact(&mut body).map_err(|e| e.to_string())?;
        let img: DiskImage = serde_json::from_slice(&body).map_err(|e| format!("镜像解析失败: {e}"))?;
        info!("曼德尔镜像已加载: {}（{} MB / 已用 {} MB）", path.display(), img.capacity_mb(), img.used_mb());
        Ok(img)
    }
}

/// 便捷函数：创建并初始化一个 .mandel 镜像
pub fn init_image(path: &Path, capacity_mb: u64) -> Result<(), String> {
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    let img = DiskImage::create(capacity_mb);
    img.save(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_list() {
        let mut img = DiskImage::create(100);
        img.mkdir("/", "docs").unwrap();
        img.write_file("/docs", "readme.txt", b"hello haavk").unwrap();
        let root = img.list_dir("/").unwrap();
        assert!(root.iter().any(|(n, d)| *n == "docs" && *d));
        let docs = img.list_dir("/docs").unwrap();
        assert!(docs.iter().any(|(n, f)| *n == "readme.txt" && !*f));
        let data = img.read_file("/docs/readme.txt").unwrap();
        assert_eq!(data, b"hello haavk");
    }

    #[test]
    fn save_and_load_roundtrip() {
        let mut img = DiskImage::create(100);
        img.mkdir("/", "etc").unwrap();
        img.write_file("/etc", "hosts", b"127.0.0.1 localhost").unwrap();
        let tmp = std::env::temp_dir().join("test_haavk.mandel");
        img.save(&tmp).unwrap();
        let loaded = DiskImage::load(&tmp).unwrap();
        assert_eq!(loaded.read_file("/etc/hosts").unwrap(), b"127.0.0.1 localhost");
        let _ = fs::remove_file(&tmp);
    }
}
