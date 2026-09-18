//! wgpu 渲染器：egui-wgpu 集成（HAAVK 桌面全部 UI 由 egui 绘制）
//!
//! 渲染管线：
//! 1. App 通过 egui Context 构建 UI，产出 `FullOutput`（shapes + textures_delta）
//! 2. 本渲染器将 shapes 曲面细分（tessellate）为 GPU 网格
//! 3. egui-wgpu 绘制到交换链表面

use std::sync::Arc;

use log::debug;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    egui_renderer: egui_wgpu::Renderer,
    size: PhysicalSize<u32>,
}

impl Renderer {
    /// 初始化 wgpu + egui 渲染器（阻塞获取 adapter/device）
    pub async fn new(window: Arc<Window>) -> Result<Self, String> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| format!("创建表面失败: {e}"))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| "未找到可用的 GPU 适配器".to_string())?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .map_err(|e| format!("创建设备失败: {e}"))?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let egui_renderer =
            egui_wgpu::Renderer::new(&device, format, None, 1, false);

        debug!("wgpu + egui 渲染器初始化完成（{}x{}）", size.width, size.height);

        Ok(Self { surface, device, queue, config, egui_renderer, size })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    /// 渲染一帧 egui UI。
    ///
    /// `ctx`：egui 上下文；`full_output`：本帧 UI 构建产物。
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        full_output: &egui::FullOutput,
    ) -> Result<(), wgpu::SurfaceError> {
        let size = self.size;
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [size.width.max(1), size.height.max(1)],
            pixels_per_point: ctx.pixels_per_point(),
        };

        // 曲面细分：egui shapes → GPU 网格
        let paint_jobs = ctx.tessellate(full_output.shapes.clone(), full_output.pixels_per_point);

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // 上传新纹理
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("haavk-encoder") });
        // 更新顶点/索引缓冲区（必须与 render 配对调用）
        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );
        {
            let mut rpass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("haavk-egui-pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.06, g: 0.08, b: 0.10, a: 1.0 }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                })
                .forget_lifetime();
            self.egui_renderer.render(&mut rpass, &paint_jobs, &screen_descriptor);
        }
        self.queue.submit(Some(encoder.finish()));

        // 释放已废弃纹理
        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        output.present();
        Ok(())
    }
}
