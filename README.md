# HAAVK OS · 曼德尔全域算力系统

> **天空哈夫克，新世界就在你耳边**
>
> HAAVK（哈夫克）全域算力系统：一套系统，跨 Android / Linux / Windows 三宿主运行。
> 内核为 **Mandel Core（曼德尔用户态微内核）**，UI 为 Windows 交互逻辑 + HAAVK 哈夫克赛博视觉。

## 架构总览

```
硬件（物理机 / QEMU 虚拟机：8核 · 16GB · 100GB · e1000）
        ↓
宿主内核（Windows NT / Linux / Android Linux —— 管理网卡、磁盘、内存）
        ↓
宿主系统（Windows 11 / Ubuntu / Android）
        ↓
【Mandel Core 曼德尔核心】   ← 本仓库
├─ Relink 同频 IPC 总线
├─ 曼德尔 VFS 虚拟文件系统
├─ 权限与资源配额（16GB 内存 / 100GB 存储）
└─ HAAVK 桌面（Windows 交互 + HAAVK 视觉）
        ↓
.hvk 哈夫克应用生态（Stage 3+）
```

## 目录结构

```
haavk-os/
├─ Cargo.toml                # workspace
├─ config/
│  └─ mandel_core.toml       # 曼德尔核心主配置
├─ assets/                   # HAAVK 徽标（PNG 多尺寸 + Windows ICO）
├─ crates/
│  ├─ mandel_core/           # 曼德尔核心（用户态微内核）
│  └─ haavk_desktop/         # HAAVK 桌面（winit + wgpu）
├─ scripts/
│  ├─ gen_icon.sh            # 徽标生成脚本（ImageMagick）
│  └─ build.sh               # 一键构建
└─ docs/
   └─ stage1.md              # 阶段一说明
```

## 快速开始

```bash
# 1. 生成徽标资源（已提交，可跳过）
bash scripts/gen_icon.sh

# 2. 构建
bash scripts/build.sh

# 3a. 无桌面环境：验证曼德尔核心
./target/debug/mandel-core --config config/mandel_core.toml --about

# 3b. 桌面环境：启动 HAAVK 桌面
./target/debug/haavk --config config/mandel_core.toml
```

## 交互（Stage 1）

| 按键 | 动作 |
| --- | --- |
| `Esc` | 断开 HAAVK 环境（退出程序，不影响宿主系统） |
| `A` | 切换「关于全域算力节点」显示模式 |

## 配置

配置文件 `config/mandel_core.toml`（运行时默认路径 `~/.haavk/config/mandel_core.toml`）：

| 配置段 | 说明 |
| --- | --- |
| `[core]` | 版本、节点 ID、日志级别、应用上限 |
| `[hardware]` | DMI 读取开关、内存/存储配额（16GB / 100GB） |
| `[relink_bus]` | Relink 同频总线（端口 42069） |
| `[vfs]` | 曼德尔虚拟文件系统（`haavk://`） |
| `[window_manager]` | 窗口管理器（渲染后端 auto） |
| `[security]` | 权限策略 |
| `[network]` | 网络代理 |

## 开发路线

| 阶段 | 内容 | 状态 |
| --- | --- | --- |
| 阶段一 | 曼德尔核心 + 基础窗口 + 硬件信息读取 | ✅ 本仓库 |
| 阶段二 | 完整桌面（窗口管理器/开始菜单/任务栏/文件管理器） | ⏳ |
| 阶段三 | `.hvk` 应用生态 + Relink 互联 + 权限系统 | ⏳ |
| 阶段四 | 跨平台完整版（Linux/Android）+ 虚拟磁盘 + 公网中继 | ⏳ |

## 里程碑：Stage 1 (MVP-1) 交付

- ✅ Mandel Core：配置解析、日志、节点 ID、DMI 硬件读取、Relink 骨架、VFS 骨架
- ✅ HAAVK 桌面窗口（winit + wgpu，HAAVK 尖塔徽标，深色赛博主题）
- ✅ 「关于全域算力节点」显示模式
- ✅ 跨平台：Linux 实测通过；Windows（含 exe 图标嵌入）与 Android 待 Stage 4 交叉编译
