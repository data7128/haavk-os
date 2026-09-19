//! 开始菜单：HAAVK 应用列表 + 搜索框 + 断开 HAAVK 环境
//!
//! 布局同 Windows：顶部 HAAVK 标识与搜索、中部应用列表、底部电源（断开环境）。

use egui::{Align, Layout, RichText};

use mandel_core::MandelCore;

use crate::app::{Action, AppKind, UiState};
use crate::theme;

const MENU_SIZE: egui::Vec2 = egui::vec2(420.0, 520.0);

/// 开始菜单应用项
const APPS: &[(&str, AppKind)] = &[
    ("全域节点", AppKind::Files),
    ("应用中心", AppKind::AppCenter),
    ("同频节点", AppKind::Nodes),
    ("全域浏览器", AppKind::Browser),
    ("系统设置", AppKind::Settings),
    ("关于 HAAVK", AppKind::About),
    ("终端", AppKind::Terminal),
];

pub fn show(ctx: &egui::Context, ui_state: &mut UiState, core: &MandelCore) -> Action {
    let mut action = Action::None;

    if !ui_state.start_open {
        return action;
    }

    // 开始菜单弹出位置：左下角（任务栏上方）
    let screen = ctx.screen_rect();
    let pos = egui::pos2(screen.left() + 10.0, screen.bottom() - 40.0 - MENU_SIZE.y);

    let resp = egui::Area::new(egui::Id::new("start_menu"))
        .fixed_pos(pos)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(theme::BG_PANEL)
                .stroke(egui::Stroke::new(1.0_f32, theme::PRIMARY_DARK))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::same(14))
                .show(ui, |ui| {
                    ui.set_width(MENU_SIZE.x);
                    ui.set_height(MENU_SIZE.y);

                    // ---------- 顶部：HAAVK 标识 + 标语 ----------
                    ui.horizontal(|ui| {
                        ui.add_space(2.0);
                        ui.label(RichText::new("HAAVK 全域入口").strong().size(17.0).color(theme::ACCENT_LIGHT));
                    });
                    ui.label(RichText::new("天空属于哈夫克，新世界就在你耳边").size(11.0).color(theme::TEXT_DIM));

                    // ---------- 搜索框 ----------
                    let search_edit = egui::TextEdit::singleline(&mut ui_state.search)
                        .hint_text("在 HAAVK 全域算力中搜索")
                        .desired_width(f32::INFINITY);
                    ui.add(search_edit);
                    ui.add_space(8.0);

                    // ---------- 应用列表（可滚动） ----------
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for (label, kind) in APPS {
                                let matched = ui_state.search.is_empty()
                                    || label.to_lowercase().contains(&ui_state.search.to_lowercase());
                                if !matched {
                                    continue;
                                }
                                let btn = egui::Button::new(
                                    RichText::new(format!("▶  {label}")).size(15.0).color(theme::TEXT_MAIN),
                                )
                                .min_size(egui::vec2(MENU_SIZE.x - 28.0, 36.0))
                                .fill(theme::BG_PANEL);
                                if ui.add(btn).clicked() {
                                    action = Action::Open(*kind);
                                    ui_state.start_open = false;
                                }
                                ui.add_space(4.0);
                            }
                            if ui_state.search.is_empty() {
                                ui.label(RichText::new(format!("节点 {} · Mandel Core {}", core.node.node_id, core.config.core.version))
                                    .size(11.0)
                                    .color(theme::TEXT_DIM));
                            }
                        });

                    // ---------- 底部：断开 HAAVK 环境 ----------
                    ui.separator();
                    ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                        let exit_btn = egui::Button::new(
                            RichText::new("⏻  断开 HAAVK 环境").size(14.0).color(theme::DANGER),
                        )
                        .fill(theme::BG_PANEL)
                        .min_size(egui::vec2(MENU_SIZE.x - 28.0, 34.0));
                        if ui.add(exit_btn).clicked() {
                            action = Action::Exit;
                            ui_state.start_open = false;
                        }
                        ui.label(
                            RichText::new("断开仅退出 HAAVK，不会关闭宿主系统")
                                .size(10.5)
                                .color(theme::TEXT_DIM),
                        );
                    });
                });
        });

    // 点击菜单外部关闭（排除开始按钮）
    let click_outside = ctx
        .input(|i| i.pointer.any_pressed())
        .then(|| ctx.input(|i| i.pointer.interact_pos()))
        .flatten();
    if let Some(p) = click_outside {
        let in_menu = resp.response.rect.contains(p);
        let in_start_btn = ui_state.start_btn_rect.map(|r| r.contains(p)).unwrap_or(false);
        if !in_menu && !in_start_btn {
            ui_state.start_open = false;
        }
    }

    action
}
