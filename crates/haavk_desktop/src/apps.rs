//! HAAVK 内置应用内容：
//! - 文件管理器（全域节点，对接曼德尔 VFS）
//! - 系统设置
//! - 关于全域算力节点
//! - 终端（Stage 3 占位）

use std::path::PathBuf;

use egui::{Align, Layout, RichText};
use mandel_core::MandelCore;

use crate::theme;

// ============================================================
// 文件管理器
// ============================================================

/// 文件管理器状态（每个文件窗口独立）
pub struct FilesState {
    pub vpath: String,
    pub history: Vec<String>,
    pub hist_idx: usize,
    pub selected: Option<String>,
}

impl FilesState {
    pub fn root() -> Self {
        Self {
            vpath: "haavk://home".into(),
            history: vec!["haavk://home".into()],
            hist_idx: 0,
            selected: None,
        }
    }

    /// 当前虚拟路径对应的宿主路径
    pub fn current_host(&self, core: &MandelCore) -> Option<PathBuf> {
        core.vfs.resolve(&self.vpath)
    }

    /// 导航到新路径（记录历史）
    pub fn navigate(&mut self, vpath: String) {
        self.history.truncate(self.hist_idx + 1);
        self.history.push(vpath.clone());
        self.hist_idx += 1;
        self.vpath = vpath;
        self.selected = None;
    }

    pub fn enter(&mut self, name: &str) {
        let base = self.vpath.trim_end_matches('/');
        self.navigate(format!("{base}/{name}"));
    }

    pub fn back(&mut self) {
        if self.hist_idx > 0 {
            self.hist_idx -= 1;
            self.vpath = self.history[self.hist_idx].clone();
            self.selected = None;
        }
    }

    pub fn forward(&mut self) {
        if self.hist_idx + 1 < self.history.len() {
            self.hist_idx += 1;
            self.vpath = self.history[self.hist_idx].clone();
            self.selected = None;
        }
    }

    pub fn can_back(&self) -> bool {
        self.hist_idx > 0
    }
    pub fn can_forward(&self) -> bool {
        self.hist_idx + 1 < self.history.len()
    }
}

/// 文件管理器 UI
pub fn files_ui(ui: &mut egui::Ui, core: &MandelCore, state: &mut FilesState) {
    // ---------- 左侧导航（全域节点挂载点） ----------
    egui::SidePanel::left("files_nav")
        .resizable(false)
        .default_width(170.0)
        .frame(egui::Frame::new().fill(theme::BG_DARK).inner_margin(egui::Margin::same(8)))
        .show_inside(ui, |ui| {
            ui.label(RichText::new("全域节点").strong().color(theme::PRIMARY));
            ui.separator();
            let mounts = core.vfs.mounts();
            for m in mounts {
                let label = m.virtual_path.trim_start_matches("haavk://");
                let is_current = state.vpath.trim_end_matches('/') == m.virtual_path;
                if ui.selectable_label(is_current, RichText::new(format!("▸  {label}")).size(13.0)).clicked() {
                    state.navigate(m.virtual_path.clone());
                }
            }
            ui.add_space(8.0);
            ui.label(RichText::new("VFS 前缀：haavk://").size(10.5).color(theme::TEXT_DIM));
        });

    // ---------- 顶部工具栏（返回/前进/地址栏） ----------
    egui::TopBottomPanel::top("files_toolbar")
        .frame(egui::Frame::new().inner_margin(egui::Margin::same(6)))
        .show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.add_enabled(state.can_back(), egui::Button::new("← 返回")).clicked() {
                    state.back();
                }
                if ui.add_enabled(state.can_forward(), egui::Button::new("→")).clicked() {
                    state.forward();
                }
                if ui.button("⟳ 刷新").clicked() {
                    state.selected = None;
                }
                if ui.button("＋ 新建文件夹").clicked() {
                    if let Some(host) = state.current_host(core) {
                        let new_dir = host.join(format!("新建文件夹_{}", chrono::Local::now().format("%H%M%S")));
                        let _ = std::fs::create_dir(&new_dir);
                    }
                }
                ui.separator();

                // 地址栏（haavk:// 协议）
                let addr = egui::TextEdit::singleline(&mut state.vpath)
                    .hint_text("haavk:// 路径")
                    .desired_width(ui.available_width() - 10.0)
                    .font(egui::TextStyle::Monospace);
                let resp = ui.add(addr);
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    state.navigate(state.vpath.clone());
                }
            });
        });

    // ---------- 目录内容 ----------
    egui::CentralPanel::default()
        .frame(egui::Frame::new().inner_margin(egui::Margin::same(8)))
        .show_inside(ui, |ui| {
            let host = state.current_host(core);
            match host {
                Some(hp) => match std::fs::read_dir(&hp) {
                    Ok(entries) => {
                        // 收集目录项
                        let mut dirs: Vec<(String, String)> = Vec::new();
                        let mut files: Vec<(String, String)> = Vec::new();
                        for e in entries.flatten() {
                            let name = e.file_name().to_string_lossy().into_owned();
                            let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
                            let meta = e.metadata().ok();
                            let mtime = meta
                                .as_ref()
                                .and_then(|m| m.modified().ok())
                                .map(|t| {
                                    let dt: chrono::DateTime<chrono::Local> = t.into();
                                    dt.format("%m-%d %H:%M").to_string()
                                })
                                .unwrap_or_else(|| "--".into());
                            if is_dir {
                                dirs.push((name, mtime));
                            } else {
                                files.push((name, mtime));
                            }
                        }
                        dirs.sort_by(|a, b| a.0.cmp(&b.0));
                        files.sort_by(|a, b| a.0.cmp(&b.0));

                        // 表格头
                        ui.horizontal(|ui| {
                            ui.add_space(30.0);
                            ui.label(RichText::new("名称").strong().size(12.5).color(theme::TEXT_DIM));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.add_space(20.0);
                                ui.label(RichText::new("修改时间").strong().size(12.0).color(theme::TEXT_DIM));
                            });
                        });
                        ui.separator();

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            // 目录
                            for (name, mtime) in &dirs {
                                if file_row(ui, state, name, true, mtime) {
                                    state.enter(name);
                                }
                            }
                            // 文件
                            for (name, mtime) in &files {
                                if file_row(ui, state, name, false, mtime) {
                                    state.selected = Some(name.clone());
                                }
                            }
                        });
                    }
                    Err(e) => {
                        ui.colored_label(theme::DANGER, format!("无法访问 {}: {e}", hp.display()));
                        if ui.button("返回全域节点根目录").clicked() {
                            state.navigate("haavk://home".into());
                        }
                    }
                },
                None => {
                    ui.colored_label(theme::DANGER, "路径无效（未挂载）");
                }
            }
        });
}

/// 文件行：图标 + 名称 + 大小 + 修改时间；双击进入目录
fn file_row(ui: &mut egui::Ui, state: &mut FilesState, name: &str, is_dir: bool, mtime: &str) -> bool {
    let selected = state.selected.as_deref() == Some(name);
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 26.0), egui::Sense::click());
    let painter = ui.painter();

    if resp.hovered() {
        painter.rect_filled(rect, 3.0, egui::Color32::from_rgba_unmultiplied(41, 200, 232, 16));
    }
    if selected {
        painter.rect_stroke(rect, 3.0, egui::Stroke::new(1.0_f32, theme::PRIMARY_DARK), egui::StrokeKind::Inside);
    }

    // 图标（目录 = 青蓝尖塔方块；文件 = 灰白文档）
    let icon_color = if is_dir { theme::PRIMARY } else { theme::TEXT_DIM };
    let ic = egui::pos2(rect.left() + 14.0, rect.center().y);
    if is_dir {
        painter.rect_filled(
            egui::Rect::from_center_size(ic, egui::vec2(13.0, 11.0)),
            2.0,
            icon_color,
        );
        theme::spire_icon(painter, ic, 8.0, theme::ACCENT_LIGHT);
    } else {
        painter.rect_filled(
            egui::Rect::from_center_size(ic, egui::vec2(9.0, 12.0)),
            1.0,
            icon_color,
        );
        painter.rect_stroke(
            egui::Rect::from_center_size(ic, egui::vec2(9.0, 12.0)),
            1.0,
            egui::Stroke::new(1.0_f32, theme::TEXT_DIM),
            egui::StrokeKind::Inside,
        );
    }

    painter.text(
        egui::pos2(rect.left() + 32.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        name,
        egui::FontId::proportional(13.5),
        if is_dir { theme::ACCENT_LIGHT } else { theme::TEXT_MAIN },
    );

    // 右侧：修改时间
    painter.text(
        egui::pos2(rect.right() - 12.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        mtime,
        egui::FontId::proportional(11.5),
        theme::TEXT_DIM,
    );

    if resp.clicked() {
        state.selected = Some(name.to_string());
    }
    resp.double_clicked() && is_dir
}

// ============================================================
// 系统设置
// ============================================================

pub fn settings_ui(ui: &mut egui::Ui, core: &MandelCore) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        // 系统信息
        egui::CollapsingHeader::new(RichText::new("系统信息").strong())
            .default_open(true)
            .show(ui, |ui| {
                kv(ui, "系统名称", "HAAVK OS · 曼德尔全域算力系统");
                kv(ui, "Mandel Core", &core.config.core.version);
                kv(ui, "节点 ID", &core.node.node_id);
                kv(ui, "日志级别", &core.config.core.log_level);
                kv(ui, "宿主内核", host_kernel_name());
                kv(ui, "标语", "天空属于哈夫克，新世界就在你耳边");
            });
        ui.separator();

        // 算力
        egui::CollapsingHeader::new(RichText::new("算力").strong())
            .default_open(true)
            .show(ui, |ui| {
                kv(ui, "硬件型号", &core.hardware.product_name);
                kv(ui, "CPU", &core.hardware.cpu);
                kv(ui, "内存", &format!("{} MB（上限 {} MB）", core.hardware.memory_mb, core.config.hardware.memory_limit_mb));
                kv(ui, "存储配额", &format!("{} MB", core.config.hardware.storage_quota_mb));
                kv(ui, "网卡", &core.hardware.network);
                kv(ui, "Relink 总线", if core.relink.enabled { "同频在线" } else { "未启用" });
            });
        ui.separator();

        // 显示
        egui::CollapsingHeader::new(RichText::new("显示").strong()).show(ui, |ui| {
            kv(ui, "渲染后端", &core.config.window_manager.render_backend);
            kv(ui, "动画", if core.config.window_manager.animations { "开" } else { "关" });
            kv(ui, "Alt+Tab 切换", if core.config.window_manager.alt_tab_switch { "开" } else { "关" });
        });
        ui.separator();

        // 个性化
        egui::CollapsingHeader::new(RichText::new("个性化").strong()).show(ui, |ui| {
            ui.label("主题强调色（视觉预览）：");
            ui.horizontal(|ui| {
                color_swatch(ui, "哈夫克青蓝", theme::PRIMARY);
                color_swatch(ui, "亮白", theme::ACCENT_LIGHT);
                color_swatch(ui, "警戒红", theme::DANGER);
            });
        });
    });
}

fn kv(ui: &mut egui::Ui, k: &str, v: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{k}：")).color(theme::TEXT_DIM));
        ui.label(RichText::new(v).monospace().color(theme::TEXT_MAIN));
    });
}

fn color_swatch(ui: &mut egui::Ui, label: &str, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(96.0, 24.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 4.0, color);
    ui.painter().text(
        egui::pos2(rect.center().x, rect.center().y),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(11.0),
        egui::Color32::BLACK,
    );
}

fn host_kernel_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "Windows NT"
    } else if cfg!(target_os = "android") {
        "Android Linux"
    } else {
        "Linux"
    }
}

// ============================================================
// 关于全域算力节点
// ============================================================

pub fn about_ui(ui: &mut egui::Ui, core: &MandelCore) {
    ui.vertical_centered(|ui| {
        ui.add_space(10.0);
        // 大徽标
        let (rect, _) = ui.allocate_exact_size(egui::vec2(96.0, 96.0), egui::Sense::hover());
        theme::spire_icon(ui.painter(), rect.center(), 88.0, theme::PRIMARY);
        ui.add_space(8.0);
        ui.label(RichText::new("HAAVK OS").strong().size(26.0).color(theme::ACCENT_LIGHT));
        ui.label(RichText::new("天空属于哈夫克，新世界就在你耳边").size(13.0).color(theme::TEXT_DIM));
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(6.0);

        let lines: Vec<String> = core.about_text().lines().map(|s| s.to_string()).collect();
        for line in &lines {
            ui.horizontal_centered(|ui| {
                ui.label(RichText::new(line).monospace().size(13.0).color(theme::TEXT_MAIN));
            });
        }
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!("Mandel Core v{} · Stage 3 (MVP-3) · .hvk 应用 + Relink 互联 + 权限系统", core.config.core.version))
                .size(11.0)
                .color(theme::TEXT_DIM),
        );
    });
}

// ============================================================
// 终端（Mandel Shell：简单命令解析）
// ============================================================

pub struct TerminalState {
    pub input: String,
    pub history: Vec<String>,
    pub cwd: String,
    pub output: Vec<String>,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            input: String::new(),
            history: vec![],
            cwd: "haavk://home".into(),
            output: vec![
                "HAAVK Mandel Shell v0.1".into(),
                "输入 help 查看可用命令".into(),
            ],
        }
    }
}

pub fn terminal_ui(ui: &mut egui::Ui, core: &MandelCore, state: &mut TerminalState) {
    ui.vertical(|ui| {
        ui.label(RichText::new("HAAVK 终端").strong().size(15.0).color(theme::ACCENT_LIGHT));
        ui.separator();

        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .max_height(ui.available_height() - 60.0)
            .show(ui, |ui| {
                for line in &state.output {
                    ui.label(RichText::new(line).monospace().size(13.0).color(theme::TEXT_MAIN));
                }
            });

        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{} $", state.cwd)).monospace().size(13.0).color(theme::PRIMARY));
            let resp = ui.add(
                egui::TextEdit::singleline(&mut state.input)
                    .desired_width(ui.available_width() - 10.0)
                    .font(egui::TextStyle::Monospace),
            );
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let cmd = state.input.trim().to_string();
                state.input.clear();
                run_shell_cmd(core, state, &cmd);
            }
        });
    });
}

fn run_shell_cmd(core: &MandelCore, state: &mut TerminalState, cmd: &str) {
    if cmd.is_empty() {
        return;
    }
    state.history.push(cmd.into());
    let mut out = |s: &str| state.output.push(s.into());

    let mut parts = cmd.split_whitespace();
    let head = parts.next().unwrap_or("");
    match head {
        "help" => {
            out("HAAVK Mandel Shell 可用命令：");
            out("  help          显示本帮助");
            out("  pwd           显示当前路径");
            out("  ls [路径]     列出目录内容");
            out("  cd <路径>     切换目录");
            out("  nodes         列出 Relink 同频节点");
            out("  apps          列出已安装 .hvk 应用");
            out("  disk          显示 .mandel 虚拟磁盘用量");
            out("  clear         清空终端");
        }
        "pwd" => out(&state.cwd),
        "ls" => {
            let path = parts.next().unwrap_or(&state.cwd);
            match core.vfs.resolve(path) {
                Some(host) => match std::fs::read_dir(&host) {
                    Ok(entries) => {
                        let names: Vec<String> = entries
                            .flatten()
                            .map(|e| e.file_name().to_string_lossy().into_owned())
                            .collect();
                        out(&format!("{}", names.join("  ")));
                    }
                    Err(e) => out(&format!("ls: 无法访问 {path}: {e}")),
                },
                None => out(&format!("ls: 未挂载路径 {path}")),
            }
        }
        "cd" => {
            if let Some(path) = parts.next() {
                state.cwd = path.to_string();
            }
        }
        "nodes" => {
            let peers = core.relink.peers();
            if peers.is_empty() {
                out("（暂无在线节点）");
            } else {
                for p in peers {
                    out(&format!("  {} @ {} v{}", p.id, p.addr, p.version));
                }
            }
        }
        "apps" => {
            out(&format!("已安装应用（{} 个）：", core.apps.count()));
            for a in core.apps.all() {
                out(&format!("  {} [{}] v{}", a.manifest.name, a.manifest.kind, a.manifest.version));
            }
        }
        "disk" => {
            if let Some(d) = &core.disk {
                out(&format!("曼德尔虚拟磁盘：{} MB / 总 {} MB", d.used_mb(), d.capacity_mb()));
            } else {
                out("虚拟磁盘未加载");
            }
        }
        "clear" => state.output.clear(),
        other => out(&format!("mandel: 未找到命令：{other}（输入 help 查看）")),
    }
}

// ============================================================
// 应用中心（.hvk 应用注册表）
// ============================================================

pub fn app_center_ui(ui: &mut egui::Ui, core: &MandelCore) {
    ui.vertical(|ui| {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("已安装应用").strong().size(16.0).color(theme::ACCENT_LIGHT));
            ui.label(RichText::new(format!("（{} 个）", core.apps.count())).size(12.0).color(theme::TEXT_DIM));
        });
        ui.label(
            RichText::new("扫描 $HAAVK_HOME/apps/ 下的 .hvk 应用包（manifest.json）+ 内置应用")
                .size(11.5)
                .color(theme::TEXT_DIM),
        );
        ui.separator();
    });

    egui::ScrollArea::vertical().show(ui, |ui| {
        let apps: Vec<_> = core.apps.all().cloned().collect();
        for app in apps {
            // 每一项：徽标方块 + 名称/版本 + 权限标签
            egui::Frame::new()
                .fill(theme::BG_PANEL)
                .inner_margin(egui::Margin::same(8))
                .corner_radius(egui::CornerRadius::same(6))
                .stroke(egui::Stroke::new(1.0_f32, theme::BORDER))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // 徽标方块
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 5.0, theme::BG_DARK);
                        theme::spire_icon(ui.painter(), rect.center(), 28.0, theme::PRIMARY);

                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&app.manifest.name).strong().size(14.0));
                                ui.label(
                                    RichText::new(format!("v{}", app.manifest.version))
                                        .size(11.0)
                                        .color(theme::TEXT_DIM),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("[{}]", app.manifest.kind))
                                        .size(10.5)
                                        .color(theme::PRIMARY),
                                );
                                ui.label(
                                    RichText::new(&app.manifest.id)
                                        .monospace()
                                        .size(11.0)
                                        .color(theme::TEXT_DIM),
                                );
                            });
                            // 权限标签
                            if !app.manifest.permissions.is_empty() {
                                ui.horizontal(|ui| {
                                    for p in &app.manifest.permissions {
                                        ui.label(
                                            RichText::new(format!("🔑 {p}"))
                                                .monospace()
                                                .size(10.0)
                                                .color(theme::PRIMARY_DARK),
                                        );
                                    }
                                });
                            }
                        });
                    });
                });
            ui.add_space(4.0);
        }
    });
}

// ============================================================
// Relink 同频节点（UDP 42069 节点发现）
// ============================================================

pub fn nodes_ui(ui: &mut egui::Ui, core: &MandelCore) {
    ui.vertical(|ui| {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("同频节点").strong().size(16.0).color(theme::ACCENT_LIGHT));
            let count = core.relink.peer_count();
            ui.label(
                RichText::new(format!("（在线 {count} 个）"))
                    .size(12.0)
                    .color(if count > 0 { theme::PRIMARY } else { theme::TEXT_DIM }),
            );
        });
        ui.label(
            RichText::new("Relink 同频总线 · UDP 广播端口 42069 · 心跳 1s / 超时 10s")
                .size(11.5)
                .color(theme::TEXT_DIM),
        );
        ui.separator();
    });

    let peers = core.relink.peers();
    if peers.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(24.0);
            ui.label(RichText::new("⌬").size(28.0).color(theme::PRIMARY));
            ui.label(RichText::new("当前局域网暂无其他 HAAVK 节点").size(13.0).color(theme::TEXT_DIM));
            ui.label(RichText::new(format!("本机节点 ID: {}", core.node.node_id)).size(11.0).color(theme::TEXT_DIM));
        });
    } else {
        egui::Grid::new("relink_peers")
            .num_columns(4)
            .striped(true)
            .show(ui, |ui| {
                ui.strong("节点 ID");
                ui.strong("地址");
                ui.strong("版本");
                ui.strong("状态");
                ui.end_row();
                for p in peers {
                    ui.label(RichText::new(&p.id).monospace().size(12.5));
                    ui.label(RichText::new(p.addr.to_string()).monospace().size(12.0).color(theme::TEXT_DIM));
                    ui.label(RichText::new(format!("v{}", p.version)).size(12.0));
                    ui.colored_label(theme::PRIMARY, "● 在线");
                    ui.end_row();
                }
            });
    }

    ui.separator();
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("本机: {} · Relink 端口 {}", core.node.node_id, core.relink.discovery_port))
            .monospace().size(11.0).color(theme::TEXT_DIM));
    });
}
