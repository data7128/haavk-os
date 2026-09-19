# HAAVK OS · 曼德尔全域算力系统

> **天空属于哈夫克，新世界就在你耳边**
>
> HAAVK（哈夫克）全域算力系统：一套系统，跨 Android / Linux / Windows 三宿主运行。
> 内核为 **Mandel Core（曼德尔用户态微内核）**，UI 为 Windows 交互逻辑 + HAAVK 哈夫克赛博视觉（青蓝 #29c8e8 / 深色主题）。

## 架构总览

```
硬件（物理机 / QEMU 虚拟机：8核 · 16GB RAM · 100GB · e1000 网卡）
        ↓
宿主内核（Windows NT / Linux / Android Linux —— 管理网卡、磁盘、内存）
        ↓
宿主系统（Windows 11 / Ubuntu / Android）
        ↓
【Mandel Core 曼德尔核心】   ← 本仓库（Rust 编写）
├─ Relink 同频 IPC 总线（UDP 42069 局域网发现 + TCP 42070 公网中继）
├─ 曼德尔 VFS 虚拟文件系统（.mandel 镜像，haavk://）
├─ 权限与资源配额（16GB 内存 / 100GB 存储 / 三级权限）
├─ .hvk 哈夫克应用包格式
└─ HAAVK 桌面（egui + wgpu，Windows 交互 + HAAVK 视觉）
        ↓
.hvk 哈夫克应用生态（文件管理器 / 终端 / 浏览器 / 设置 / 节点面板…）
```

## 目录结构

```
haavk-os/
├─ Cargo.toml                # workspace
├─ config/
│  └─ mandel_core.toml       # 曼德尔核心主配置
├─ assets/                   # HAAVK 徽标（PNG 多尺寸 + Windows ICO + 中文字体）
├─ crates/
│  ├─ mandel_core/           # 曼德尔核心（用户态微内核）
│  │  ├─ src/lib.rs          # 主系统集成
│  │  ├─ src/config.rs      # 配置解析
│  │  ├─ src/hardware.rs     # 硬件/DMI/IP 读取
│  │  ├─ src/relink.rs       # UDP 发现 + TCP 中继客户端（自动重连）
│  │  ├─ src/vfs.rs          # 曼德尔 VFS
│  │  ├─ src/apps.rs         # .hvk 应用包 + 应用仓库
│  │  ├─ src/permissions.rs  # 三级权限模型
│  │  ├─ src/disk.rs         # .mandel 虚拟磁盘镜像
│  │  └─ src/bin/relay_server.rs  # 公网中继服务端
│  └─ haavk_desktop/         # HAAVK 桌面（winit + wgpu + egui）
│     ├─ src/main.rs         # 入口
│     ├─ src/app.rs          # 事件循环 + 窗口管理
│     ├─ src/theme.rs        # HAAVK 配色/字体/徽标
│     ├─ src/desktop.rs      # 壁纸 + 图标 + 右键菜单
│     ├─ src/taskbar.rs      # 任务栏 + 通知中心
│     ├─ src/start_menu.rs   # 开始菜单
│     └─ src/apps.rs         # 文件管理器/终端/浏览器/设置/节点面板
├─ scripts/
│  └─ gen_icon.sh            # 徽标生成脚本
└─ docs/
   ├─ stage1.md              # 阶段一说明
   ├─ stage4.md              # 阶段四说明
   └─ boot_log.txt           # 启动日志样本
```

## 快速开始

```bash
# 1. 构建
cargo build -p haavk_desktop

# 2a. 无桌面环境：验证曼德尔核心
./target/debug/mandel-core --config config/mandel_core.toml --about

# 2b. 桌面环境：启动 HAAVK 桌面
./target/debug/haavk --config config/mandel_core.toml

# 3. 公网中继服务端（另一台机器上运行）
./target/debug/relay-server --port 42070
```

## 已实现功能

### 内核 & 硬件
- ✅ Mandel Core 曼德尔用户态微内核（Rust）
- ✅ 100GB `.mandel` 虚拟磁盘镜像（JSON inode 树，魔数 HAAVK-MANDEL-V1）
- ✅ `.hvk` 应用包格式 + AppRegistry 应用仓库
- ✅ 三级权限模型（Restricted / Standard / Trusted）
- ✅ 硬件信息采集：CPU / 内存 / 本机 IP（UDP connect 获取）
- ✅ DMI 硬件铭牌定制

### 网络
- ✅ UDP 42069 局域网节点自动发现（Hello 广播 + 10 秒超时清理）
- ✅ TCP 42070 公网中继服务端（多节点并发 + Hello/AppMessage 转发）
- ✅ RelayClient 后台自动重连（指数退避 1s→30s）
- ✅ Mandel Shell：`ping`（TCP 连通测试）/ `curl`（HTTP GET）
- ✅ 系统设置网络面板（本机 IP / Relink 状态 / 在线节点数）

### 桌面 UI
- ✅ 开机动画（2.2 秒 HAAVK 徽标 + 标语 + 进度条）
- ✅ 任务栏 + 开始菜单（7 个内置应用入口）+ 通知中心
- ✅ 桌面右键菜单（9 项：打开各应用/刷新/断开）
- ✅ 窗口管理（拖动/最小化/最大化/关闭，对齐 Windows 交互）
- ✅ Alt+Tab 窗口切换
- ✅ 壁纸切换
- ✅ 赛博青蓝视觉主题（#29c8e8 主色 + 深色背景）

### 内置应用
- ✅ **全域浏览器**：多标签页 + 地址栏 + HTTP 抓取 + HTML 纯文本渲染 + 书签 + 历史记录
- ✅ **Mandel Shell 终端**：`help/pwd/ls/cd/nodes/apps/disk/clear/ping/curl`
- ✅ 全域节点（文件管理器，对接曼德尔 VFS）
- ✅ 应用中心 / 同频节点面板
- ✅ 系统设置（算力/网络/显示）
- ✅ 关于 HAAVK

## 配置

配置文件 `config/mandel_core.toml`：

| 配置段 | 说明 |
| --- | --- |
| `[core]` | 版本、节点 ID、日志级别、应用上限 |
| `[hardware]` | DMI 读取、内存 16GB / 存储 100GB 配额 |
| `[relink_bus]` | Relink 同频总线（端口 42069） |
| `[vfs]` | 曼德尔虚拟文件系统（`haavk://`） |
| `[window_manager]` | 窗口管理器（渲染后端 auto） |
| `[security]` | 权限策略 |
| `[network]` | 网络代理 / 中继服务器地址 |

## 开发路线

| 阶段 | 内容 | 状态 |
| --- | --- | --- |
| 阶段一 | 曼德尔核心 + 基础窗口 + 硬件信息读取 | ✅ |
| 阶段二 | 完整桌面（窗口管理器/开始菜单/任务栏/文件管理器） | ✅ |
| 阶段三 | `.hvk` 应用生态 + Relink 互联 + 权限系统 | ✅ |
| 阶段四 | 虚拟磁盘 + 公网中继 + 跨平台构建文档 | ✅ |
| 阶段五 | 细节打磨：开机动画/通知中心/文件操作/Mandel Shell | ✅ |
| 阶段六 | 内置浏览器（HTTP 抓取）+ 中继服务端 | ✅ |
| 阶段七 | 浏览器多标签 + 中继自动重连 + 右键菜单完善 | ✅ |
| 阶段八 | 浏览器书签/历史 + README 完善 | ✅ |

## 待开发

- ⏳ 浏览器：CSS 富渲染 / JavaScript 引擎
- ⏳ 中继消息可视化面板
- ⏳ Windows exe / Android APK 交叉编译产物（需 mingw-w64 / Android NDK）
