//! 桌面层：壁纸 + 桌面图标（双击打开）+ 右键菜单
//!
//! 交互逻辑同 Windows：桌面图标单击选中、双击打开；右键弹出 HAAVK 系统菜单。

use egui::Sense;

use crate::app::{Action, AppKind, UiState};
use crate::theme;

/// 桌面图标项
const DESKTOP_ICONS: &[(&str, AppKind)] = &[
    ("全域节点", AppKind::Files),
    ("系统设置", AppKind::Settings),
    ("关于 HAAVK", AppKind::About),
    ("终端", AppKind::Terminal),
];

pub fn show(ctx: &egui::Context, ui_state: &mut UiState) -> Action {
    let mut action = Action::None;

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(theme::BG_DARK))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            // 壁纸
            theme::paint_wallpaper(ui.painter(), rect);

            // 顶部标语
            ui.painter().text(
                egui::pos2(rect.right() - 14.0, rect.top() + 12.0),
                egui::Align2::RIGHT_TOP,
                "天空属于哈夫克，新世界就在你耳边",
                egui::FontId::proportional(15.0),
                theme::TEXT_DIM,
            );

            // 桌面图标区（左上角）
            egui::Area::new(egui::Id::new("desktop_icons"))
                .fixed_pos(egui::pos2(18.0, 18.0))
                .order(egui::Order::Background)
                .show(ctx, |ui| {
                    ui.set_max_width(200.0);
                    egui::Grid::new("desktop_icons_grid")
                        .num_columns(2)
                        .spacing(egui::vec2(10.0, 12.0))
                        .show(ui, |ui| {
                            for (label, kind) in DESKTOP_ICONS {
                                if icon_button(ui, label, *kind, ui_state.accent) {
                                    action = Action::Open(*kind);
                                }
                                ui.end_row();
                            }
                        });
                });

            // 桌面右键：记录菜单位置
            if ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Secondary)) {
                if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                    ui_state.menu_pos = Some(pos);
                }
            }
        });

    // 右键菜单
    if let Some(pos) = ui_state.menu_pos {
        let mut close = false;
        let resp = egui::Area::new(egui::Id::new("desktop_menu"))
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.set_min_width(170.0);
                ui.label(egui::RichText::new("HAAVK 系统菜单").strong().color(theme::PRIMARY));
                ui.separator();
                if ui.button("打开全域节点").clicked() {
                    action = Action::Open(AppKind::Files);
                    close = true;
                }
                if ui.button("全域浏览器").clicked() {
                    action = Action::Open(AppKind::Browser);
                    close = true;
                }
                if ui.button("应用中心").clicked() {
                    action = Action::Open(AppKind::AppCenter);
                    close = true;
                }
                if ui.button("同频节点").clicked() {
                    action = Action::Open(AppKind::Nodes);
                    close = true;
                }
                if ui.button("终端").clicked() {
                    action = Action::Open(AppKind::Terminal);
                    close = true;
                }
                if ui.button("系统设置").clicked() {
                    action = Action::Open(AppKind::Settings);
                    close = true;
                }
                if ui.button("关于 HAAVK").clicked() {
                    action = Action::Open(AppKind::About);
                    close = true;
                }
                if ui.button("刷新").clicked() {
                    close = true;
                }
                ui.separator();
                if ui.button("断开 HAAVK 环境").clicked() {
                    action = Action::Exit;
                    close = true;
                }
            });
        ui_state.menu_rect = Some(resp.response.rect);

        // 点击菜单外区域关闭
        let click_outside = ctx.input(|i| i.pointer.any_pressed()).then(|| {
            ctx.input(|i| i.pointer.interact_pos())
        });
        if let Some(Some(p)) = click_outside {
            if !resp.response.rect.contains(p) {
                close = true;
            }
        }
        if close {
            ui_state.menu_pos = None;
        }
    }

    action
}

/// 桌面图标按钮：尖塔图标 + 名称（单击选中、双击打开）
fn icon_button(ui: &mut egui::Ui, label: &str, _kind: AppKind, accent: egui::Color32) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(86.0, 96.0), Sense::click());
    let painter = ui.painter();

    // 选中高亮
    if resp.hovered() || resp.is_pointer_button_down_on() {
        painter.rect_filled(rect.shrink(2.0), 6.0, egui::Color32::from_rgba_unmultiplied(41, 200, 232, 26));
    }
    if resp.is_pointer_button_down_on() {
        painter.rect_stroke(rect.shrink(2.0), 6.0, egui::Stroke::new(1.0_f32, theme::PRIMARY_DARK), egui::StrokeKind::Inside);
    }

    // 尖塔图标
    theme::spire_icon(painter, egui::pos2(rect.center().x, rect.top() + 26.0), 40.0, accent);

    // 名称
    let text_color = if resp.hovered() { theme::ACCENT_LIGHT } else { theme::TEXT_MAIN };
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 60.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(14.0),
        text_color,
    );

    resp.double_clicked()
}
