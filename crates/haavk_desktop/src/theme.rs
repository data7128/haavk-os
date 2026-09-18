//! HAAVK 主题：哈夫克赛博视觉（深色磨砂 · 青蓝发光 · 硬朗工业风）
//!
//! - 背景深色：`#0f1419`
//! - 强调色：哈夫克青蓝 `#29c8e8`
//! - 文字：浅灰白；选中：哈夫克蓝
//! - 控件：半透明亚克力磨砂、细发光边框

use std::sync::Arc;

use egui::{
    Color32, Context, FontData, FontDefinitions, FontFamily, FontTweak, Style, Visuals,
};

// ---------- HAAVK 配色 ----------
pub const PRIMARY: Color32 = Color32::from_rgb(41, 200, 232); // 哈夫克青蓝
pub const PRIMARY_DARK: Color32 = Color32::from_rgb(18, 150, 176);
pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(216, 249, 255); // 亮青
pub const BG_DARK: Color32 = Color32::from_rgb(15, 20, 25); // 桌面底色
pub const BG_PANEL: Color32 = Color32::from_rgb(21, 28, 36); // 面板
pub const BG_TASKBAR: Color32 = Color32::from_rgb(13, 17, 23); // 任务栏
pub const TEXT_MAIN: Color32 = Color32::from_rgb(214, 226, 235);
pub const TEXT_DIM: Color32 = Color32::from_rgb(120, 136, 150);
pub const BORDER: Color32 = Color32::from_rgb(35, 48, 60);
pub const DANGER: Color32 = Color32::from_rgb(229, 87, 87);

// 中文字体（Noto Sans CJK SC，ttc index=2）
const CJK_FONT_PATHS: &[&str] = &[
    "assets/fonts/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    "C:\\Windows\\Fonts\\msyh.ttc", // Windows 微软雅黑
];

/// 应用 HAAVK 主题（字体 + 视觉样式）
pub fn setup(ctx: &Context) {
    install_fonts(ctx);
    install_style(ctx);
}

fn install_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    // 查找可用的中文字体文件
    let mut cjk: Option<Vec<u8>> = None;
    for p in CJK_FONT_PATHS {
        if let Ok(bytes) = std::fs::read(p) {
            log::info!("加载中文字体: {p}");
            cjk = Some(bytes);
            break;
        }
    }

    if let Some(bytes) = cjk {
        fonts
            .font_data
            .insert("haavk_cjk".into(), Arc::new(FontData { font: bytes.into(), index: 2, tweak: FontTweak::default() }));
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            fonts.families.entry(family).or_default().push("haavk_cjk".into());
        }
    } else {
        log::warn!("未找到中文字体，中文可能显示异常");
    }

    ctx.set_fonts(fonts);
}

fn install_style(ctx: &Context) {
    let mut style: Style = (*ctx.style()).clone();
    style.visuals = Visuals::dark();

    // 基础
    style.visuals.panel_fill = BG_PANEL;
    style.visuals.window_fill = BG_PANEL;
    style.visuals.extreme_bg_color = BG_DARK;
    style.visuals.faint_bg_color = Color32::from_rgb(18, 25, 33);
    style.visuals.selection.bg_fill = PRIMARY;
    style.visuals.selection.stroke = egui::Stroke::new(1.0_f32, ACCENT_LIGHT);
    style.visuals.hyperlink_color = PRIMARY;
    style.visuals.override_text_color = Some(TEXT_MAIN);
    style.visuals.warn_fg_color = Color32::from_rgb(240, 173, 78);
    style.visuals.error_fg_color = DANGER;

    // 控件
    for w in [
        &mut style.visuals.widgets.noninteractive,
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
    ] {
        w.bg_fill = BG_PANEL;
        w.weak_bg_fill = Color32::from_rgb(24, 32, 42);
        w.fg_stroke = egui::Stroke::new(1.0_f32, TEXT_MAIN);
        w.bg_stroke = egui::Stroke::new(1.0_f32, BORDER);
        w.corner_radius = egui::CornerRadius::same(4);
    }
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, PRIMARY_DARK);
    style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(30, 42, 54);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(28, 40, 52);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, PRIMARY);
    style.visuals.widgets.open.bg_fill = Color32::from_rgb(26, 36, 46);
    style.visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0_f32, PRIMARY);

    // 窗口
    style.visuals.window_stroke = egui::Stroke::new(1.0_f32, PRIMARY_DARK);
    style.visuals.window_corner_radius = egui::CornerRadius::same(6);
    style.visuals.window_shadow =
        egui::Shadow { offset: [0, 8], blur: 24, spread: 0, color: Color32::from_black_alpha(120) };

    // 间距
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.window_margin = egui::Margin::same(10);

    // 滚动条
    style.spacing.scroll = egui::style::ScrollStyle::solid();

    ctx.set_style(style);
}

/// HAAVK 官方几何徽标（painter 绘制：三竖条 + 斜向箭头菱形）
/// 图形：左窄竖条 / 中高竖条 / 右中竖条 + 右侧向右箭头
pub fn spire_icon(painter: &egui::Painter, center: egui::Pos2, size: f32, color: Color32) {
    let h = size;
    let bw = size * 0.18; // 竖条宽
    let gap = size * 0.07;
    let left_x = center.x - size * 0.33;

    // 左竖条（较矮）
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(left_x, center.y - h * 0.26),
            egui::pos2(left_x + bw, center.y + h * 0.30),
        ),
        0.0,
        color,
    );
    // 中竖条（最高，主塔）
    let mid_x = left_x + bw + gap;
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(mid_x, center.y - h * 0.5),
            egui::pos2(mid_x + bw, center.y + h * 0.5),
        ),
        0.0,
        color,
    );
    // 右竖条（中高）
    let right_x = mid_x + bw + gap;
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(right_x, center.y - h * 0.20),
            egui::pos2(right_x + bw, center.y + h * 0.30),
        ),
        0.0,
        color,
    );
    // 右侧向右箭头菱形
    let ax = right_x + bw;
    painter.add(egui::Shape::convex_polygon(
        vec![
            egui::pos2(ax, center.y - h * 0.14),
            egui::pos2(ax + size * 0.17, center.y + h * 0.04),
            egui::pos2(ax, center.y + h * 0.22),
        ],
        color,
        egui::Stroke::NONE,
    ));
}

/// 哈夫克尖塔壁纸：深色渐变天空 + 尖塔剪影 + 网格（程序化绘制）
pub fn paint_wallpaper(painter: &egui::Painter, rect: egui::Rect) {
    // 垂直渐变
    let steps = 48;
    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let y0 = rect.top() + rect.height() * t;
        let y1 = rect.top() + rect.height() * (t + 1.0) / steps as f32;
        let color = Color32::from_rgb(
            12 + (22.0 * t) as u8,
            18 + (30.0 * t) as u8,
            28 + (34.0 * t) as u8,
        );
        painter.rect_filled(
            egui::Rect::from_min_max(egui::pos2(rect.left(), y0), egui::pos2(rect.right(), y1)),
            0.0,
            color,
        );
    }

    // 地平线发光
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(rect.left(), rect.bottom() - 90.0),
            egui::pos2(rect.right(), rect.bottom()),
        ),
        0.0,
        Color32::from_rgba_unmultiplied(23, 168, 198, 26),
    );

    // 城市天际线剪影（随机高度塔楼，确定性伪随机）
    let mut seed = 12345u32;
    let mut next = move || {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (seed >> 16) as f32 / 65535.0
    };
    let base = rect.bottom() - 60.0;
    let mut x = rect.left();
    while x < rect.right() {
        let w = 26.0 + next() * 40.0;
        let h = 30.0 + next() * 130.0;
        painter.rect_filled(
            egui::Rect::from_min_max(egui::pos2(x, base - h), egui::pos2(x + w, base)),
            0.0,
            Color32::from_rgb(12, 17, 24),
        );
        // 窗口亮点
        for _ in 0..3 {
            let wx = x + 4.0 + next() * (w - 8.0);
            let wy = base - h + 4.0 + next() * (h - 10.0);
            painter.rect_filled(
                egui::Rect::from_min_max(egui::pos2(wx, wy), egui::pos2(wx + 2.5, wy + 3.5)),
                0.0,
                Color32::from_rgba_unmultiplied(41, 200, 232, 90),
            );
        }
        x += w + 2.0;
    }

    // 中央哈夫克尖塔（高塔 + 顶部标志区）
    let spire_w = 34.0;
    let spire_top = base - 260.0;
    painter.rect_filled(
        egui::Rect::from_min_max(egui::pos2(rect.center().x - spire_w / 2.0, spire_top), egui::pos2(rect.center().x + spire_w / 2.0, base)),
        0.0,
        Color32::from_rgb(10, 14, 20),
    );
    // 尖塔顶部发光点
    painter.circle_filled(egui::pos2(rect.center().x, spire_top - 6.0), 4.0, PRIMARY);

    // 网格线（远景）
    let grid = Color32::from_rgba_unmultiplied(41, 200, 232, 10);
    for gx in (rect.left() as i64..rect.right() as i64).step_by(64) {
        painter.line_segment(
            [egui::pos2(gx as f32, rect.top()), egui::pos2(gx as f32, rect.bottom())],
            egui::Stroke::new(1.0_f32, grid),
        );
    }
}
