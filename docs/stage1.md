# 阶段一（Stage 1 · MVP-1）：曼德尔核心 + 基础窗口

> 目标：让 HAAVK 以最小形态跑起来 —— 曼德尔核心启动、读取配置与宿主硬件信息、弹出 HAAVK 桌面窗口。

## 交付清单

| 组件 | 状态 | 说明 |
| --- | --- | --- |
| `mandel_core` 核心库 | ✅ | 配置解析 / 日志 / 节点 ID / DMI 硬件读取 |
| `mandel-core` CLI | ✅ | 无桌面环境下验证核心启动与「关于」信息 |
| `haavk_desktop` 桌面 | ✅ | winit + wgpu 窗口、HAAVK 徽标渲染、关于模式 |
| `mandel_core.toml` | ✅ | 核心主配置（16GB 内存 / 100GB 存储配额） |
| HAAVK 徽标 | ✅ | 尖塔 + 曼德尔菱形，PNG 多尺寸 + Windows ICO |
| 构建脚本 | ✅ | `scripts/build.sh`，图标生成 `scripts/gen_icon.sh` |

## 曼德尔核心能力（本阶段）

1. **配置加载**：`config/mandel_core.toml`，缺失时回退内置默认配置
2. **日志系统**：控制台 + 文件落盘（`~/.haavk/logs/haavk.log`）
3. **节点 ID**：配置指定 > 持久化 `~/.haavk/node_id` > 自动生成
4. **DMI 硬件读取**：
   - Linux：`/sys/class/dmi/id/*` + `/proc/cpuinfo` + `/proc/meminfo` + 网卡枚举
   - Windows：PowerShell `Get-CimInstance`（Win32_ComputerSystemProduct 等）
5. **Relink 总线**：骨架（状态初始化，UDP 42069 留给 Stage 3）
6. **曼德尔 VFS**：骨架（`haavk://` 挂载表：system / home / downloads / documents / pictures）

## 桌面窗口能力（本阶段）

- winit 窗口（1280×800，标题 "HAAVK OS · 曼德尔全域算力终端"）
- 窗口图标 = HAAVK 尖塔徽标（嵌入式 PNG）
- wgpu 渲染：深色赛博背景 + 尖塔徽标（发光外圈 + 尖塔三角 + 曼德尔菱形）+ 底部任务栏雏形 + HAAVK 开始按钮
- `A` 键切换「关于」模式：背景提亮、徽标变白、窗口标题显示节点信息
- `Esc` / 关闭窗口：断开 HAAVK 环境（不影响宿主系统）

## 验证方式

```bash
# 1. 核心自检（无 GUI 环境）
./target/debug/mandel-core --config config/mandel_core.toml --about

# 2. 单元测试
cargo test -p mandel_core

# 3. 桌面（需图形环境）
./target/debug/haavk --config config/mandel_core.toml
```

## 下一步（阶段二）

- 窗口管理器：拖拽 / 缩放 / 最小化最大化关闭 / Alt+Tab（Windows 交互逻辑）
- 开始菜单 + 任务栏完整交互
- 曼德尔 VFS 对接 HAAVK 文件管理器（全域节点）
- 字体渲染引入，完整「关于」面板 UI
