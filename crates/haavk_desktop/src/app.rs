//! HAAVK 桌面应用：winit 事件循环 + egui-winit 集成 + 桌面 UI 状态
//!
//! 交互（Stage 2）：
//! - `Esc` 断开 HAAVK 环境（退出程序，不影响宿主）
//! - `Alt+Tab` 在应用窗口间切换
//! - 桌面图标双击打开应用；任务栏/开始菜单管理窗口

use std::sync::Arc;
use std::time::Instant;

use log::{info, warn};
use mandel_core::MandelCore;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::Key;
use winit::window::{Icon, Window, WindowId};

use crate::apps;
use crate::desktop;
use crate::renderer::Renderer;
use crate::start_menu;
use crate::taskbar;
use crate::theme;

/// 嵌入式 HAAVK 徽标（256px PNG，编译期打入二进制）
const HAAVK_ICON_PNG: &[u8] = include_bytes!("../../../assets/haavk_256.png");

/// 应用类型
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppKind {
    Files,
    Settings,
    About,
    Terminal,
    AppCenter,
    Nodes,
}

impl AppKind {
    pub fn title(&self) -> &'static str {
        match self {
            AppKind::Files => "HAAVK 文件管理器 - 全域节点",
            AppKind::Settings => "HAAVK 系统设置",
            AppKind::About => "关于全域算力节点",
            AppKind::Terminal => "HAAVK 终端",
            AppKind::AppCenter => "HAAVK 应用中心",
            AppKind::Nodes => "Relink 同频节点",
        }
    }
}

/// 一个应用窗口
pub struct AppWindow {
    pub id: u64,
    pub kind: AppKind,
    pub open: bool,
    pub minimized: bool,
    pub files: Option<apps::FilesState>,
    pub terminal: Option<apps::TerminalState>,
}

/// 桌面 UI 全局状态
pub struct UiState {
    pub start_open: bool,
    pub start_btn_rect: Option<egui::Rect>,
    pub search: String,
    pub menu_pos: Option<egui::Pos2>,
    pub menu_rect: Option<egui::Rect>,
    pub windows: Vec<AppWindow>,
    pub next_id: u64,
    pub active: Option<u64>,
    pub accent: egui::Color32,
    pub alt_tab_hint: Option<(u64, Instant)>,
}

impl UiState {
    fn new() -> Self {
        Self {
            start_open: false,
            start_btn_rect: None,
            search: String::new(),
            menu_pos: None,
            menu_rect: None,
            windows: Vec::new(),
            next_id: 1,
            active: None,
            accent: theme::PRIMARY,
            alt_tab_hint: None,
        }
    }

    /// 打开一个应用（已存在则激活并还原，否则新建）
    fn open_app(&mut self, kind: AppKind) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.kind == kind && w.open) {
            w.minimized = false;
            self.active = Some(w.id);
            return;
        }
        if let Some(w) = self.windows.iter_mut().find(|w| w.kind == kind) {
            // 已关闭的同类窗口：重新打开
            w.open = true;
            w.minimized = false;
            self.active = Some(w.id);
            return;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.windows.push(AppWindow {
            id,
            kind,
            open: true,
            minimized: false,
            files: if kind == AppKind::Files { Some(apps::FilesState::root()) } else { None },
            terminal: if kind == AppKind::Terminal { Some(apps::TerminalState::default()) } else { None },
        });
        self.active = Some(id);
        info!("打开应用窗口：{}", kind.title());
    }

    fn toggle_minimize(&mut self, id: u64) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.minimized = !w.minimized;
            if w.minimized && self.active == Some(id) {
                // 激活下一个可见窗口
                self.active = self
                    .windows
                    .iter()
                    .find(|x| x.id != id && x.open && !x.minimized)
                    .map(|x| x.id);
            }
        }
    }

    fn activate(&mut self, id: u64) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id && w.open) {
            w.minimized = false;
            self.active = Some(id);
        }
    }
}

/// UI 构建动作（desktop/taskbar/start_menu 输出，App 统一执行）
#[derive(Default)]
pub enum Action {
    #[default]
    None,
    Open(AppKind),
    ToggleStart,
    Activate(u64),
    ToggleMinimize(u64),
    Exit,
}

pub struct App {
    core: MandelCore,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    egui_ctx: egui::Context,
    egui_state: Option<egui_winit::State>,
    ui: UiState,
}

impl App {
    pub fn new(core: MandelCore) -> Self {
        Self {
            core,
            window: None,
            renderer: None,
            egui_ctx: egui::Context::default(),
            egui_state: None,
            ui: UiState::new(),
        }
    }

    pub fn run(mut self) -> Result<(), String> {
        let event_loop = EventLoop::new().map_err(|e| format!("创建事件循环失败: {e}"))?;
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self).map_err(|e| format!("事件循环运行失败: {e}"))
    }

    /// 创建 HAAVK 主窗口 + 渲染器 + egui 状态
    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let attrs = Window::default_attributes()
            .with_title("HAAVK OS · 曼德尔全域算力终端")
            .with_inner_size(PhysicalSize::new(1280, 800))
            .with_min_inner_size(PhysicalSize::new(900, 620));

        let window = Arc::new(event_loop.create_window(attrs).map_err(|e| format!("创建窗口失败: {e}"))?);
        self.window = Some(window.clone());

        // 窗口图标（HAAVK 尖塔徽标）
        match decode_icon() {
            Ok(icon) => window.set_window_icon(Some(icon)),
            Err(e) => warn!("窗口图标加载失败: {e}"),
        }

        // wgpu + egui 渲染器
        match pollster::block_on(Renderer::new(window.clone())) {
            Ok(r) => {
                self.renderer = Some(r);
                info!("HAAVK 桌面渲染器已初始化");
            }
            Err(e) => {
                warn!("GPU 初始化失败，进入无渲染模式: {e}");
            }
        }

        // egui-winit 状态
        let egui_state = egui_winit::State::new(
            self.egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &*window,
            None,
            None,
            None,
        );
        self.egui_state = Some(egui_state);

        // HAAVK 主题
        theme::setup(&self.egui_ctx);
        info!("HAAVK 桌面窗口已创建（1280x800）");
        Ok(())
    }

    /// 渲染一帧
    fn render_frame(&mut self) {
        let window_arc = self.window.clone();
        let Some(window) = window_arc.as_ref() else { return };

        // 暂时取出 egui 状态，避免与 UI 构建闭包冲突
        let mut state_opt = self.egui_state.take();
        let Some(state) = state_opt.as_mut() else {
            self.egui_state = state_opt;
            return;
        };

        let raw = state.take_egui_input(window);
        let ctx = self.egui_ctx.clone();
        let full = ctx.run(raw, |ctx| self.build_ui(ctx));
        state.handle_platform_output(window, full.platform_output.clone());

        if let Some(r) = &mut self.renderer {
            if let Err(e) = r.render(&self.egui_ctx, &full) {
                match e {
                    wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => {
                        r.resize(window.inner_size());
                    }
                    other => warn!("渲染错误: {other:?}"),
                }
            }
        }

        self.egui_state = state_opt;
    }

    /// 构建桌面 UI（桌面 → 窗口 → 任务栏 → 开始菜单）
    fn build_ui(&mut self, ctx: &egui::Context) {
        self.handle_alt_tab(ctx);

        // 1. 桌面（最底层）
        let d_action = desktop::show(ctx, &mut self.ui);
        self.apply(d_action);

        // 2. 应用窗口
        self.show_windows(ctx);

        // 3. 任务栏（底部）
        let t_action = taskbar::show(ctx, &mut self.ui);
        self.apply(t_action);

        // 4. 开始菜单
        let s_action = start_menu::show(ctx, &mut self.ui, &self.core);
        self.apply(s_action);
    }

    /// 应用窗口层
    fn show_windows(&mut self, ctx: &egui::Context) {
        // 先收集需要关闭的窗口（从后往前删）
        let mut close_ids: Vec<u64> = Vec::new();

        // 渲染每个可见窗口
        let mut i = 0;
        while i < self.ui.windows.len() {
            let is_open = self.ui.windows[i].open;
            let is_min = self.ui.windows[i].minimized;
            if !is_open {
                close_ids.push(self.ui.windows[i].id);
                i += 1;
                continue;
            }
            if is_min {
                i += 1;
                continue;
            }

            let id = self.ui.windows[i].id;
            let kind = self.ui.windows[i].kind;
            let is_active = self.ui.active == Some(id);

            let mut open = true;
            let title = kind.title();
            let inner = egui::Window::new(title)
                .id(egui::Id::new(("haavk_app", id)))
                .open(&mut open)
                .default_size([760.0, 520.0])
                .min_size([440.0, 320.0])
                .show(ctx, |ui| match kind {
                    AppKind::Files => {
                        let st = self.ui.windows[i].files.get_or_insert_with(apps::FilesState::root);
                        apps::files_ui(ui, &self.core, st);
                    }
                    AppKind::Settings => apps::settings_ui(ui, &self.core),
                    AppKind::About => apps::about_ui(ui, &self.core),
                    AppKind::Terminal => {
                        let st = self.ui.windows[i].terminal.get_or_insert_with(apps::TerminalState::default);
                        apps::terminal_ui(ui, &self.core, st);
                    }
                    AppKind::AppCenter => apps::app_center_ui(ui, &self.core),
                    AppKind::Nodes => apps::nodes_ui(ui, &self.core),
                });

            self.ui.windows[i].open = open;

            if let Some(inner) = inner {
                let response = inner.response;
                // 点击窗口区域 = 激活
                if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                    if response.rect.contains(pos) && ctx.input(|i| i.pointer.any_pressed()) {
                        self.ui.active = Some(id);
                    }
                }
            }
            if is_active && !open {
                self.ui.active = None;
            }

            i += 1;
        }

        // 删除已关闭窗口
        for cid in &close_ids {
            if self.ui.active == Some(*cid) {
                self.ui.active = None;
            }
            if let Some(pos) = self.ui.windows.iter().position(|w| w.id == *cid) {
                self.ui.windows.remove(pos);
            }
        }
        // 若活动窗口失效，选第一个可见窗口
        if self.ui.active.is_none() {
            self.ui.active = self
                .ui
                .windows
                .iter()
                .find(|w| w.open && !w.minimized)
                .map(|w| w.id);
        }
    }

    /// Alt+Tab 窗口切换
    fn handle_alt_tab(&mut self, ctx: &egui::Context) {
        let (alt, tab) = ctx.input(|i| (i.modifiers.alt, i.key_pressed(egui::Key::Tab)));
        if alt && tab {
            let ids: Vec<u64> = self
                .ui
                .windows
                .iter()
                .filter(|w| w.open && !w.minimized)
                .map(|w| w.id)
                .collect();
            if !ids.is_empty() {
                let idx = ids
                    .iter()
                    .position(|&x| self.ui.active == Some(x))
                    .unwrap_or(0);
                let next = ids[(idx + 1) % ids.len()];
                self.ui.active = Some(next);
                self.ui.alt_tab_hint = Some((next, Instant::now()));
            }
        }
    }

    /// 执行 UI 动作
    fn apply(&mut self, action: Action) {
        match action {
            Action::None => {}
            Action::Open(kind) => self.ui.open_app(kind),
            Action::ToggleStart => {
                self.ui.start_open = !self.ui.start_open;
            }
            Action::Activate(id) => self.ui.activate(id),
            Action::ToggleMinimize(id) => self.ui.toggle_minimize(id),
            Action::Exit => {
                self.core.shutdown();
                self.egui_ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            if let Err(e) = self.create_window(event_loop) {
                warn!("窗口创建失败: {e}");
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // 先交给 egui-winit 处理输入
        if let (Some(state), Some(window)) = (&mut self.egui_state, &self.window) {
            let _ = state.on_window_event(&*window, &event);
        }

        match event {
            WindowEvent::CloseRequested => {
                self.core.shutdown();
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(r) = &mut self.renderer {
                    r.resize(size);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Esc 断开 HAAVK 环境
                if event.logical_key == Key::Named(winit::keyboard::NamedKey::Escape)
                    && event.state == winit::event::ElementState::Pressed
                {
                    self.core.shutdown();
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => self.render_frame(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // 用 WaitUntil 平滑驱动帧率，避免 Poll 空转吃满 CPU
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            std::time::Instant::now() + std::time::Duration::from_millis(16),
        ));
        // 持续请求重绘（软件渲染 60fps，保证动画/交互稳定刷新）
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

/// 解码嵌入 PNG → winit 窗口图标
fn decode_icon() -> Result<Icon, String> {
    let img = image::load_from_memory(HAAVK_ICON_PNG).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Icon::from_rgba(rgba.into_raw(), w, h).map_err(|e| e.to_string())
}
