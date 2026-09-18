# 阶段二（Stage 2 · MVP-2）：完整桌面环境

> 目标：把 HAAVK 做成一个「像 Windows 一样操作、却是 HAAVK 品牌」的完整桌面 —— 窗口管理器、开始菜单、任务栏、文件管理器（全域节点）、系统设置、关于、Alt+Tab。

## 交付清单

| 组件 | 状态 | 说明 |
| --- | --- | --- |
| egui 0.31 桌面框架 | ✅ | egui + egui-wgpu 0.31 + wgpu 24 全套 UI（与阶段一 wgpu 24 兼容的唯一组合） |
| 中文字体 | ✅ | Noto Sans CJK SC（ttc 内嵌，Windows 回退 msyh.ttc） |
| 桌面（壁纸/图标/右键菜单） | ✅ | 程序化深色赛博壁纸 + 算力波形 + 4 应用图标 + HAAVK 系统菜单 |
| 任务栏 | ✅ | HAAVK 开始按钮（尖塔徽标）+ 应用窗口按钮 + 托盘（时钟/日期/算力在线） |
| 开始菜单 | ✅ | HAAVK 全域入口 + 搜索框 + 应用列表 + 节点信息 + 「断开 HAAVK 环境」 |
| 文件管理器 | ✅ | HAAVK 文件管理器 - 全域节点，VFS `haavk://` 协议，真实目录浏览 |
| 系统设置 | ✅ | 系统信息 / 算力（CPU、内存配额、存储配额、网卡、Relink） |
| 关于 HAAVK | ✅ | 尖塔徽标 + 标语 + 版本 |
| Alt+Tab / 窗口切换 | ✅ | 激活窗口切换（Windows 交互逻辑） |
| 终端（占位） | ✅ | 预留应用框架 |

## 技术要点（阶段二关键决策）

1. **UI 框架升级**：阶段一自绘顶点 → 阶段二 **egui 0.31 + egui-wgpu 0.31 + egui-winit 0.31**。
   - 版本探测结论：egui-wgpu 0.31 ↔ wgpu 24 是唯一匹配组合（0.33→wgpu 27，0.36→wgpu 30）。
2. **渲染管线三件套**：每帧必须 `update_texture`（字体/图片纹理）→ `update_buffers`（顶点/索引缓冲）→ `render`（绘制），三者缺一不可（漏 `update_buffers` 会 panic）。
3. **`forget_lifetime`**：egui-wgpu 0.31 的 `render` 要求 `RenderPass<'static>`，wgpu 24 提供 `RenderPass::forget_lifetime()` 转换。
4. **窗口生命周期**：`self.window` 必须在 `create_window` 中赋值（`Arc` 克隆），否则事件循环照常跑但桌面永不渲染（本次修复的核心 bug）。
5. **事件驱动**：`ControlFlow::WaitUntil(16ms)` + 每帧 `request_redraw`，避免 Poll 空转吃满 CPU；软件渲染（无 GPU）下 60fps 稳定。
6. **字体**：`FontData` 需 `Arc` 包裹（egui 0.31）；ttc 多字重用 `index: 2` 选中 SC。

## 桌面交互（Windows 习惯 + HAAVK 品牌）

| Windows 概念 | HAAVK 对应 |
| --- | --- |
| 桌面图标 | 全域节点 / 系统设置 / 关于 HAAVK / 终端 |
| 桌面右键菜单 | HAAVK 系统菜单（打开全域节点/设置/关于/刷新/断开环境） |
| 任务栏开始按钮 | HAAVK 尖塔按钮（点击开合开始菜单） |
| 开始菜单 | HAAVK 全域入口（搜索 + 应用列表 + 节点信息） |
| 文件资源管理器 | HAAVK 文件管理器 - 全域节点（`haavk://home`） |
| 关机 | 断开 HAAVK 环境（仅退出程序，不关宿主） |
| 通知 | （阶段三：Relink 同频消息） |
| Alt+Tab | 激活窗口切换 |

## 验证方式

```bash
# 1. 编译（零警告）
cargo build -p haavk_desktop

# 2. 单元测试（核心 9 项）
cargo test

# 3. 实机验证（图形环境）
./target/release/haavk --config config/mandel_core.toml
```

实测（Xvfb + Openbox 虚拟显示器，1440×900）：
- 桌面渲染：深色壁纸 + 青色网格 + 算力波形数据可视化 ✅
- 双击「全域节点」→ 文件管理器打开，`haavk://home` 列出真实主目录（.agents/.config/.haavk 等 + 真实修改时间）✅
- 开始按钮 → HAAVK 全域入口（搜索框/应用列表/节点 HAAVK-NODE-00001/断开环境）✅
- 开始菜单 → 系统设置：系统信息 + 算力（CPU Intel Xeon 8457C、内存 4033MB(上限16384MB)、存储配额 102400MB、网卡 eth1、Relink 同频在线）✅
- 开始菜单 → 关于 HAAVK：尖塔徽标 + 「天空哈夫克，新世界就在你耳边」✅
- 桌面右键 → HAAVK 系统菜单 ✅
- Alt+Tab 切换激活窗口 ✅
- 验证截图：`docs/screenshots/stage2_*.png`

## 已修复的关键问题（本次排障记录）

1. `self.window` 未赋值 → 桌面永不渲染（黑屏）：`create_window` 中补 `self.window = Some(window.clone())`。
2. egui-wgpu 缺 `update_buffers` → `Option::unwrap()` panic：补调用。
3. `RenderPass` 生命周期 → `forget_lifetime()`。
4. Poll 空转 CPU 100% → `WaitUntil(16ms)`。
5. 无 WM 的 Xvfb 下窗口 Occluded、request_redraw 被抑制 → 验证环境配 Openbox。

## 下一步（阶段三）

- `.hvk` 应用格式 + 应用沙箱
- Relink 互联完整化（UDP 42069 节点发现、消息、权限系统）
- 通知中心（Relink 同频消息）
