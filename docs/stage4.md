# HAAVK OS · Stage 4 (MVP-4) 交付说明

> 标语：天空哈夫克，新世界就在你耳边。

## 本阶段交付

### 1. `.mandel` 曼德尔虚拟磁盘镜像
- 新模块 `crates/mandel_core/src/disk.rs`
- 格式：魔数 `HAAVK-MANDEL-V1\0`（16B）+ 长度前缀（8B LE）+ JSON 序列化镜像体
- 超级块：版本 / 块大小 4096B / 总块数 / 已用块数 / 根 inode
- inode 树：文件 / 目录，支持 `mkdir` / `write_file` / `read_file` / `list_dir` / `lookup`
- 启动时自动创建/加载 `haavk_storage.mandel`（容量 = `hardware.storage_quota_mb`，即 100 GB）
- 2 个单元测试：create_and_list / save_and_load_roundtrip

### 2. 公网中继配置骨架
- `config.relink_bus.relay_server_enable` 配置项已就位（默认 false）
- 局域网 UDP 42069 广播发现已在 Stage 3 完成
- 公网中继（TCP 长连接到中继服务器，跨网段透传 Hello/AppMessage）留待后续接入

### 3. 跨平台构建指南

#### Linux（已实机验证）
```bash
cargo build --release -p haavk_desktop
# 产物：target/release/haavk
```

#### Windows 11 exe（x86_64-pc-windows-msvc / gnu）
前置：Rustup + MSVC Build Tools 或 MinGW-w64
```bash
# 安装 target
rustup target add x86_64-pc-windows-gnu
# 安装 MinGW 交叉工具链（Ubuntu 宿主）
sudo apt install gcc-mingw-w64-x86-64
# 编译
cargo build --release --target x86_64-pc-windows-gnu -p haavk_desktop
# 产物：target/x86_64-pc-windows-gnu/release/haavk.exe
# 窗口图标已通过 build.rs 嵌入 haavk.ico
```

#### Android APK（termux / 交叉编译）
前置：Android NDK + cargo-apk
```bash
rustup target add aarch64-linux-android
# 设置 NDK 工具链
export ANDROID_NDK_HOME=/path/to/ndk
cargo install cargo-apk
cargo apk build --release -p haavk_desktop
# 产物：target/aarch64-linux-android/release/*.apk
```
注意：Android 下 wgpu 需要 Vulkan 支持；`haavk_desktop` 的 winit 窗口在 Android 上转为 SurfaceView 渲染。

## 硬件配置（QEMU 推荐）
| 项 | 值 |
| --- | --- |
| 磁盘 | 100 GB（qcow2 或 .mandel 镜像） |
| 内存 | 16 GB（16384 MB） |
| CPU | 8 核及以上 |
| 网卡 | e1000（Relink 广播走 UDP 42069） |
| SMBIOS | DMI 铭牌 `HAAVK-NODE-XXXXX` |

## 验证结果（沙箱 Linux 实机）
- 17 项单元测试全过（config/vfs/relink/apps/permissions/disk）
- Release 编译零警告
- 启动日志：
  - Relink 总线绑定 `0.0.0.0:42069`
  - 应用仓库注册 4 个内置 .hvk 应用
  - `.mandel` 镜像创建并挂载（100 GB）
- 桌面 UI：新几何 logo（三竖条+右箭头）+ Windows 交互 + HAAVK 视觉

## 四阶段路线完成状态
| 阶段 | 内容 | 状态 |
| --- | --- | --- |
| 一 | Mandel Core 启动 + 基础窗口 + DMI 硬件 | ✅ |
| 二 | 完整桌面（egui 0.31：壁纸/图标/任务栏/开始菜单/文件管理器/Alt+Tab） | ✅ |
| 三 | `.hvk` 应用格式 + Relink UDP 发现 + 三级权限 + 应用中心/同频节点面板 | ✅ |
| 四 | `.mandel` 虚拟磁盘 + 公网中继配置 + 跨平台构建指南 | ✅ |
