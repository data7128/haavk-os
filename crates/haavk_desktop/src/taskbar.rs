//! 任务栏：开始按钮 + 应用窗口按钮 + 系统托盘（时钟 / 算力状态）
//!
//! 交互同 Windows：点击应用按钮切换（激活/最小化）；托盘显示时间与算力状态。

use chrono::Local;
use egui::{Align, Layout, RichText, Sense, StrokeKind};

use crate::app::{Action, UiState};
use crate::theme;

pub fn show(ctx: &egui::Context, ui_state: &mut UiState) -> Action {
    let mut action = Action::None;
    let active = ui_state.active;

    egui::TopBottomPanel::bottom("haavk_taskbar")
        .frame(
            egui::Frame::new()
                .fill(theme::BG_TASKBAR)
                .inner_margin(egui::Margin::symmetric(8, 5))
                .stroke(egui::Stroke::new(1.0_f32, theme::BORDER)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // ---------- 开始按钮（HAAVK 尖塔） ----------
                let (start_rect, start_resp) =
                    ui.allocate_exact_size(egui::vec2(46.0, 38.0), Sense::click());
                let painter = ui.painter();
                painter.rect_stroke(
                    start_rect.shrink(1.0),
                    4.0,
                    egui::Stroke::new(
                        1.5_f32,
                        if ui_state.start_open { theme::ACCENT_LIGHT } else { theme::PRIMARY_DARK },
                    ),
                    StrokeKind::Inside,
                );
                theme::spire_icon(painter, start_rect.center(), 22.0, ui_state.accent);
                if start_resp.clicked() {
                    action = Action::ToggleStart;
                }
                // 记录开始按钮区域（供开始菜单外部点击关闭检测）
                ui_state.start_btn_rect = Some(start_rect);

                ui.separator();

                // ---------- 应用窗口按钮 ----------
                let open_windows: Vec<(u64, &str, bool)> = ui_state
                    .windows
                    .iter()
                    .filter(|w| w.open)
                    .map(|w| (w.id, w.kind.title(), active == Some(w.id)))
                    .collect();
                for (id, title, is_active) in open_windows {
                    let btn = ui.selectable_label(
                        is_active,
                        RichText::new(title).size(13.0).color(if is_active { theme::ACCENT_LIGHT } else { theme::TEXT_MAIN }),
                    );
                    if btn.clicked() {
                        // Windows 行为：点击激活；已激活则最小化
                        if is_active {
                            action = Action::ToggleMinimize(id);
                        } else {
                            action = Action::Activate(id);
                        }
                    }
                }

                // ---------- 托盘（右对齐） ----------
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    // 时钟
                    let now = Local::now();
                    let time = now.format("%H:%M:%S").to_string();
                    let date = now.format("%m-%d 周%a").to_string().replace("周", "周");
                    ui.vertical(|ui| {
                        ui.label(RichText::new(time).size(14.0).monospace().strong().color(theme::ACCENT_LIGHT));
                        ui.label(RichText::new(date).size(11.0).color(theme::TEXT_DIM));
                    });
                    ui.separator();

                    // 通知中心铃铛
                    let bell = egui::Button::new(
                        RichText::new("🔔").size(14.0).color(if ui_state.notifications_open { theme::ACCENT_LIGHT } else { theme::TEXT_MAIN }),
                    )
                    .min_size(egui::vec2(32.0, 32.0))
                    .fill(if ui_state.notifications_open { theme::PRIMARY_DARK } else { theme::BG_TASKBAR });
                    if ui.add(bell).clicked() {
                        ui_state.notifications_open = !ui_state.notifications_open;
                    }
                    ui.add_space(6.0);

                    // 算力状态（Relink）
                    ui.label(RichText::new("▲ 算力在线").size(12.0).color(theme::PRIMARY));
                    ui.add_space(6.0);
                });
            });
        });

    // 通知中心弹出面板
    if ui_state.notifications_open {
        let screen = ctx.screen_rect();
        let pos = egui::pos2(screen.right() - 320.0, screen.bottom() - 40.0 - 360.0);
        egui::Area::new(egui::Id::new("notif_center"))
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(theme::BG_PANEL)
                    .stroke(egui::Stroke::new(1.0_f32, theme::PRIMARY_DARK))
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_width(300.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🔔 通知中心").strong().size(15.0).color(theme::ACCENT_LIGHT));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.small_button("清空").clicked() {
                                    ui_state.notifications.clear();
                                }
                            });
                        });
                        ui.separator();
                        if ui_state.notifications.is_empty() {
                            ui.label(RichText::new("暂无通知").color(theme::TEXT_DIM));
                        } else {
                            for n in &ui_state.notifications {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("●").color(theme::PRIMARY));
                                    ui.label(RichText::new(n).size(12.0).color(theme::TEXT_MAIN));
                                });
                                ui.separator();
                            }
                        }
                    });
            });
    }

    action
}
