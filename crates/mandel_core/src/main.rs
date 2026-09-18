//! mandel-core CLI：无桌面环境下的曼德尔核心自检入口。
//!
//! 用法：
//!   mandel-core [--config <path>]    指定配置文件
//!   mandel-core --about              打印「关于全域算力节点」信息后退出
//!   mandel-core --help               帮助

use std::env;
use std::path::Path;

use mandel_core::MandelCore;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut config_path: Option<String> = None;
    let mut show_about = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--config" => {
                i += 1;
                config_path = args.get(i).cloned();
            }
            "--about" => show_about = true,
            "--help" | "-h" => {
                print_help();
                return;
            }
            other => {
                eprintln!("未知参数: {other}");
                print_help();
                std::process::exit(2);
            }
        }
        i += 1;
    }

    let cfg = config_path.as_deref().map(Path::new);
    let core = match MandelCore::boot(cfg) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[HAAVK] 曼德尔核心启动失败: {e}");
            std::process::exit(1);
        }
    };

    if show_about {
        println!();
        println!("----------------------------------------");
        println!("{}", core.about_text());
        println!("----------------------------------------");
    }

    core.shutdown();
}

fn print_help() {
    println!(
        "mandel-core — HAAVK 曼德尔核心 CLI\n\
         \n\
         用法:\n\
         \x20 mandel-core [--config <path>] [--about]\n\
         \n\
         选项:\n\
         \x20 --config <path>  指定 mandel_core.toml 路径\n\
         \x20 --about          打印「关于全域算力节点」信息\n\
         \x20 -h, --help       显示帮助"
    );
}
