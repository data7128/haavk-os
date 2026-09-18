//! HAAVK Desktop 入口
//!
//! 启动流程：
//! 1. 启动 Mandel Core（曼德尔核心，加载配置/节点/硬件/Relink/VFS）
//! 2. 创建 HAAVK 桌面窗口（winit + wgpu，窗口图标为 HAAVK 尖塔徽标）
//! 3. 进入渲染循环；`Esc` 断开 HAAVK 环境（不影响宿主系统）

use std::path::Path;

use log::info;
use mandel_core::MandelCore;

use crate::app::App;

mod app;
mod apps;
mod desktop;
mod renderer;
mod start_menu;
mod taskbar;
mod theme;

fn main() {
    // 配置路径：命令行 `--config <path>`，缺省用默认路径
    let mut config_path: Option<String> = None;
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--config" => {
                i += 1;
                config_path = args.get(i).cloned();
            }
            "--help" | "-h" => {
                println!(
                    "haavk — HAAVK OS 曼德尔全域算力终端\n\
                     用法: haavk [--config <path>]\n\
                     \x20 Esc      断开 HAAVK 环境\n\
                     \x20 Alt+Tab  应用窗口切换\n\
                     \x20 桌面图标双击打开应用\n\
                     \x20 任务栏/开始菜单管理窗口"
                );
                return;
            }
            other => {
                eprintln!("未知参数: {other}");
                std::process::exit(2);
            }
        }
        i += 1;
    }

    let core = match MandelCore::boot(config_path.as_deref().map(Path::new)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[HAAVK] 曼德尔核心启动失败: {e}");
            std::process::exit(1);
        }
    };
    info!("Mandel Core 就绪，进入 HAAVK 桌面……");

    let app = App::new(core);
    if let Err(e) = app.run() {
        eprintln!("[HAAVK] 桌面运行失败: {e}");
        std::process::exit(1);
    }
}
