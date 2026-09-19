# HAAVK Mandel Bare Metal Kernel 构建指南

## 环境要求

- Rust nightly（`rustup default nightly`）
- QEMU（`qemu-system-x86_64`）
- bootimage（`cargo install bootimage`）
- x86_64 目标（`rustup target add x86_64-bootimage`）

## 构建步骤

```bash
# 1. 安装 nightly 工具链
rustup default nightly
rustup target add x86_64-bootimage

# 2. 安装 bootimage
cargo install bootimage

# 3. 构建内核
cargo build -p haavk_kernel --target x86_64-bootimage

# 4. 用 QEMU 启动
qemu-system-x86_64 -drive format=raw,file=target/x86_64-bootimage/debug/haavk_kernel
```

## 内核启动流程

```
BIOS/UEFI
   ↓
bootloader（GRUB / syslinux）
   ↓
_start()（本内核入口，0x100000）
   ↓
VGA 文本模式初始化（0xB8000 显存，80x25）
   ↓
打印 HAAVK 标志 + 内核版本
   ↓
hlt 指令停机
```

## 当前状态

- ✅ no_std / no_main 裸机入口
- ✅ VGA 文本模式驱动（0xB8000 显存直接写入）
- ✅ HAAVK 标志 + 标语打印
- ✅ 内核 panic 处理
- ⏳ GDT / IDT 中断表
- ⏳ 物理内存管理（paging）
- ⏳ e1000 网卡驱动
- ⏳ .mandel 虚拟磁盘挂载
- ⏳ 从裸机内核启动 Mandel Core 用户态服务

## 与用户态桌面的关系

```
裸机内核（haavk_kernel）          ← 本阶段新增
   ↓ 启动用户态
Mandel Core（mandel_core）        ← 已有
   ↓
HAAVK 桌面（haavk_desktop）       ← 已有
```
